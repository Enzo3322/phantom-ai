#![allow(deprecated)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[derive(Clone, Debug, PartialEq)]
pub enum AudioSource {
    Mic,
    System,
    Both,
}

impl AudioSource {
    pub fn from_str(s: &str) -> Self {
        match s {
            "mic" => Self::Mic,
            "system" => Self::System,
            _ => Self::Both,
        }
    }
}

pub fn start_capture(
    source: AudioSource,
    audio_tx: mpsc::Sender<Vec<f32>>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    if source == AudioSource::Mic || source == AudioSource::Both {
        let tx = audio_tx.clone();
        let flag = stop_flag.clone();
        std::thread::spawn(move || {
            if let Err(e) = run_mic_capture(tx, flag) {
                eprintln!("[phantom] mic capture error: {e}");
            }
        });
    }

    if source == AudioSource::System || source == AudioSource::Both {
        let tx = audio_tx.clone();
        let flag = stop_flag.clone();
        std::thread::spawn(move || {
            if let Err(e) = capture_system_audio(tx, flag) {
                eprintln!("[phantom] system audio error: {e}");
            }
        });
    }

    Ok(())
}

fn run_mic_capture(
    audio_tx: mpsc::Sender<Vec<f32>>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;

    eprintln!("[phantom] mic device: {}", device.name().unwrap_or_default());

    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get input config: {e}"))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    eprintln!("[phantom] mic config: {}Hz, {} channels", sample_rate, channels);

    let resampler = if sample_rate != 16000 {
        Some(AudioResampler::new(sample_rate, 16000))
    } else {
        None
    };

    let resampler = Arc::new(std::sync::Mutex::new(resampler));
    let flag = stop_flag.clone();

    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if flag.load(Ordering::Relaxed) {
                    return;
                }

                let mono = to_mono(data, channels);

                let samples = {
                    let mut res = resampler.lock().unwrap_or_else(|e| e.into_inner());
                    match res.as_mut() {
                        Some(r) => r.process(&mono),
                        None => mono,
                    }
                };

                if !samples.is_empty() {
                    let _ = audio_tx.send(samples);
                }
            },
            |err| eprintln!("[phantom] mic stream error: {err}"),
            None,
        )
        .map_err(|e| format!("Failed to build input stream: {e}"))?;

    stream.play().map_err(|e| format!("Failed to start mic stream: {e}"))?;

    eprintln!("[phantom] mic capture started");

    // Keep the stream alive until stop signal
    while !stop_flag.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // stream is dropped here, stopping capture
    eprintln!("[phantom] mic capture stopped");
    Ok(())
}

fn to_mono(data: &[f32], channels: u16) -> Vec<f32> {
    if channels == 1 {
        return data.to_vec();
    }
    let ch = channels as usize;
    data.chunks_exact(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

struct AudioResampler {
    ratio: f64,
    buffer: Vec<f32>,
    position: f64,
}

impl AudioResampler {
    fn new(from_rate: u32, to_rate: u32) -> Self {
        Self {
            ratio: to_rate as f64 / from_rate as f64,
            buffer: Vec::new(),
            position: 0.0,
        }
    }

    fn process(&mut self, input: &[f32]) -> Vec<f32> {
        self.buffer.extend_from_slice(input);

        let mut output = Vec::new();
        while (self.position as usize) < self.buffer.len().saturating_sub(1) {
            let idx = self.position as usize;
            let frac = self.position - idx as f64;
            let sample = self.buffer[idx] * (1.0 - frac as f32)
                + self.buffer[idx + 1] * frac as f32;
            output.push(sample);
            self.position += 1.0 / self.ratio;
        }

        let consumed = self.position as usize;
        if consumed > 0 && consumed <= self.buffer.len() {
            self.buffer.drain(..consumed);
            self.position -= consumed as f64;
        }

        output
    }
}

#[cfg(target_os = "macos")]
fn capture_system_audio(
    audio_tx: mpsc::Sender<Vec<f32>>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    use cocoa::base::{id, nil, YES, NO};
    use objc::{class, msg_send, sel, sel_impl};
    use objc::runtime::{Class, Object};
    use std::os::raw::c_void;

    unsafe {
        let sc_content_class = Class::get("SCShareableContent")
            .ok_or("SCShareableContent not available (requires macOS 13+)")?;

        let (done_tx, done_rx) = mpsc::channel::<Result<id, String>>();
        let done_tx_ptr = Box::into_raw(Box::new(done_tx));

        let block = block::ConcreteBlock::new(move |content: id, error: id| {
            let tx = &*done_tx_ptr;
            if error != nil {
                let desc: id = msg_send![error, localizedDescription];
                let cstr: *const std::os::raw::c_char = msg_send![desc, UTF8String];
                let err_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                let _ = tx.send(Err(err_str));
            } else {
                let _: () = msg_send![content, retain];
                let _ = tx.send(Ok(content));
            }
        });
        let block = block.copy();

        let _: () = msg_send![sc_content_class,
            getShareableContentExcludingDesktopWindows: NO
            onScreenWindowsOnly: YES
            completionHandler: &*block
        ];

        let content = done_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "Timeout getting shareable content".to_string())?
            .map_err(|e| format!("Failed to get shareable content: {e}"))?;

        let displays: id = msg_send![content, displays];
        let display_count: usize = msg_send![displays, count];
        if display_count == 0 {
            return Err("No displays available for system audio capture".to_string());
        }
        let display: id = msg_send![displays, objectAtIndex: 0usize];

        let config_class = Class::get("SCStreamConfiguration")
            .ok_or("SCStreamConfiguration not available")?;
        let config: id = msg_send![config_class, alloc];
        let config: id = msg_send![config, init];

        let _: () = msg_send![config, setCapturesAudio: YES];
        let _: () = msg_send![config, setExcludesCurrentProcessAudio: YES];
        let _: () = msg_send![config, setSampleRate: 16000i32];
        let _: () = msg_send![config, setChannelCount: 1i32];
        let _: () = msg_send![config, setWidth: 2usize];
        let _: () = msg_send![config, setHeight: 2usize];

        let filter_class = Class::get("SCContentFilter")
            .ok_or("SCContentFilter not available")?;
        let empty_arr: id = msg_send![class!(NSArray), array];
        let filter: id = msg_send![filter_class, alloc];
        let filter: id = msg_send![filter,
            initWithDisplay: display
            excludingWindows: empty_arr
        ];

        // Register delegate class
        let delegate_class = register_audio_delegate();
        let delegate: id = msg_send![delegate_class, alloc];
        let delegate: id = msg_send![delegate, init];

        let callback_data = Box::new(AudioCallbackData {
            tx: audio_tx.clone(),
            stop_flag: stop_flag.clone(),
        });
        let callback_ptr = Box::into_raw(callback_data);

        (*(delegate as *mut Object)).set_ivar("callbackData", callback_ptr as *mut c_void);

        let stream_class = Class::get("SCStream")
            .ok_or("SCStream not available")?;
        let stream: id = msg_send![stream_class, alloc];
        let stream: id = msg_send![stream,
            initWithFilter: filter
            configuration: config
            delegate: nil
        ];

        let queue_label = std::ffi::CString::new("com.phantom.audio").unwrap();
        let queue = dispatch_queue_create(queue_label.as_ptr(), std::ptr::null());

        let mut error: id = nil;
        let sc_stream_output_type_audio: i64 = 1;
        let _: () = msg_send![stream,
            addStreamOutput: delegate
            type: sc_stream_output_type_audio
            sampleHandlerQueue: queue
            error: &mut error
        ];

        if error != nil {
            let desc: id = msg_send![error, localizedDescription];
            let cstr: *const std::os::raw::c_char = msg_send![desc, UTF8String];
            let err_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
            return Err(format!("Failed to add stream output: {err_str}"));
        }

        // Start capture
        let (start_tx, start_rx) = mpsc::channel::<Result<(), String>>();
        let start_tx_ptr = Box::into_raw(Box::new(start_tx));

        let start_block = block::ConcreteBlock::new(move |error: id| {
            let tx = &*start_tx_ptr;
            if error != nil {
                let desc: id = msg_send![error, localizedDescription];
                let cstr: *const std::os::raw::c_char = msg_send![desc, UTF8String];
                let err_str = std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string();
                let _ = tx.send(Err(err_str));
            } else {
                let _ = tx.send(Ok(()));
            }
        });
        let start_block = start_block.copy();

        let _: () = msg_send![stream, startCaptureWithCompletionHandler: &*start_block];

        start_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "Timeout starting system audio capture".to_string())?
            .map_err(|e| format!("Failed to start capture: {e}"))?;

        eprintln!("[phantom] system audio capture started");

        while !stop_flag.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Stop capture
        let (stop_tx_ch, stop_rx_ch) = mpsc::channel::<()>();
        let stop_tx_ptr = Box::into_raw(Box::new(stop_tx_ch));

        let stop_block = block::ConcreteBlock::new(move |_error: id| {
            let tx = &*stop_tx_ptr;
            let _ = tx.send(());
        });
        let stop_block = stop_block.copy();

        let _: () = msg_send![stream, stopCaptureWithCompletionHandler: &*stop_block];
        let _ = stop_rx_ch.recv_timeout(std::time::Duration::from_secs(3));

        let _ = Box::from_raw(callback_ptr);

        eprintln!("[phantom] system audio capture stopped");
        Ok(())
    }
}

#[cfg(target_os = "macos")]
struct AudioCallbackData {
    tx: mpsc::Sender<Vec<f32>>,
    stop_flag: Arc<AtomicBool>,
}

#[cfg(target_os = "macos")]
extern "C" {
    fn dispatch_queue_create(
        label: *const std::os::raw::c_char,
        attr: *const std::os::raw::c_void,
    ) -> cocoa::base::id;
}

#[cfg(target_os = "macos")]
fn register_audio_delegate() -> &'static objc::runtime::Class {
    use cocoa::base::id;
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Protocol, Sel};
    use objc::{sel, sel_impl};
    use std::os::raw::c_void;
    use std::sync::Once;

    static REGISTER: Once = Once::new();
    static mut CLASS: Option<&'static Class> = None;

    REGISTER.call_once(|| {
        let superclass = Class::get("NSObject").unwrap();
        let mut decl = ClassDecl::new("PhantomAudioDelegate", superclass).unwrap();

        decl.add_ivar::<*mut c_void>("callbackData");

        extern "C" fn stream_did_output_sample_buffer(
            this: &Object,
            _sel: Sel,
            _stream: id,
            sample_buffer: id,
            output_type: i64,
        ) {
            if output_type != 1 {
                return;
            }

            unsafe {
                let ptr: *mut c_void = *this.get_ivar("callbackData");
                if ptr.is_null() {
                    return;
                }
                let data = &*(ptr as *const AudioCallbackData);

                if data.stop_flag.load(Ordering::Relaxed) {
                    return;
                }

                if let Some(samples) = extract_audio_samples(sample_buffer) {
                    let _ = data.tx.send(samples);
                }
            }
        }

        unsafe {
            decl.add_method(
                sel!(stream:didOutputSampleBuffer:ofType:),
                stream_did_output_sample_buffer
                    as extern "C" fn(&Object, Sel, id, id, i64),
            );

            if let Some(protocol) = Protocol::get("SCStreamOutput") {
                decl.add_protocol(protocol);
            }

            CLASS = Some(decl.register());
        }
    });

    unsafe { CLASS.unwrap() }
}

#[cfg(target_os = "macos")]
unsafe fn extract_audio_samples(sample_buffer: cocoa::base::id) -> Option<Vec<f32>> {
    extern "C" {
        fn CMSampleBufferGetDataBuffer(sbuf: cocoa::base::id) -> cocoa::base::id;
        fn CMBlockBufferGetDataLength(block_buffer: cocoa::base::id) -> usize;
        fn CMBlockBufferGetDataPointer(
            block_buffer: cocoa::base::id,
            offset: usize,
            length_at_offset: *mut usize,
            total_length: *mut usize,
            data_pointer: *mut *mut u8,
        ) -> i32;
    }

    let block_buffer = CMSampleBufferGetDataBuffer(sample_buffer);
    if block_buffer.is_null() {
        return None;
    }

    let data_len = CMBlockBufferGetDataLength(block_buffer);
    if data_len == 0 {
        return None;
    }

    let mut data_ptr: *mut u8 = std::ptr::null_mut();
    let mut length: usize = 0;
    let mut total: usize = 0;

    let status = CMBlockBufferGetDataPointer(
        block_buffer,
        0,
        &mut length,
        &mut total,
        &mut data_ptr,
    );

    if status != 0 || data_ptr.is_null() {
        return None;
    }

    let float_count = total / std::mem::size_of::<f32>();
    let float_ptr = data_ptr as *const f32;
    let samples = std::slice::from_raw_parts(float_ptr, float_count).to_vec();

    Some(samples)
}

#[cfg(not(target_os = "macos"))]
fn capture_system_audio(
    _audio_tx: mpsc::Sender<Vec<f32>>,
    _stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    Err("System audio capture is only supported on macOS".to_string())
}
