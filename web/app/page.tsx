import { Animations, HeroGlow } from "./animations";

export default function Home() {
  return (
    <>
      <Animations />
      <HeroGlow />

      <section id="hero">
        <div className="hero-glow" aria-hidden="true" />
        <nav>
          <span className="logo">Phantom</span>
          <div className="nav-links">
            <a href="#features">Features</a>
            <a href="#stealth-layers">Stealth</a>
            <a href="#how-it-works">How it works</a>
            <a href="#download">Download</a>
          </div>
        </nav>
        <div className="hero-content">
          <h1 data-animate="fade-up">
            AI-powered screenshot analysis.
            <br />
            Invisible to screen capture.
          </h1>
          <p className="hero-sub" data-animate="fade-up" data-delay="1">
            Press a shortcut, get instant AI answers in a floating overlay that
            no one else can see.
          </p>
          <a
            href="/downloads/Phantom.dmg"
            className="btn-download"
            data-animate="fade-up"
            data-delay="2"
          >
            Download for macOS
          </a>
          <span className="hero-note" data-animate="fade-up" data-delay="3">
            macOS 12+ &middot; Apple Silicon &middot; Free
          </span>
        </div>
      </section>

      <section id="features">
        <h2 data-animate="fade-up">Core Features</h2>
        <div className="features-grid">
          <div className="feature-card" data-animate="scale-in" data-delay="0">
            <div className="feature-icon">&#128373;</div>
            <h3>Stealth Mode</h3>
            <p>
              The overlay is invisible to screenshots and screen recordings. Uses
              macOS private APIs to hide from any screen capture.
            </p>
          </div>
          <div className="feature-card" data-animate="scale-in" data-delay="1">
            <div className="feature-icon">&#9889;</div>
            <h3>Instant Analysis</h3>
            <p>
              Press <kbd>&#8984;&#8679;S</kbd> to capture your screen and get an
              AI response in seconds.
            </p>
          </div>
          <div className="feature-card" data-animate="scale-in" data-delay="2">
            <div className="feature-icon">&#127908;</div>
            <h3>Voice Transcription</h3>
            <p>
              Record audio with <kbd>&#8984;&#8679;M</kbd> and get instant
              AI-powered transcription via Whisper.
            </p>
          </div>
          <div className="feature-card" data-animate="scale-in" data-delay="3">
            <div className="feature-icon">&#129302;</div>
            <h3>Multiple AI Models</h3>
            <p>
              Choose between Gemini 2.0 Flash, 2.5 Flash, 2.5 Pro, and 3.1 Pro.
            </p>
          </div>
          <div className="feature-card" data-animate="scale-in" data-delay="4">
            <div className="feature-icon">&#9000;</div>
            <h3>Keyboard-First</h3>
            <p>
              Global shortcuts work from any app. No dock icon, no distractions.
            </p>
          </div>
          <div className="feature-card" data-animate="scale-in" data-delay="5">
            <div className="feature-icon">&#128736;</div>
            <h3>Customizable</h3>
            <p>
              Custom prompts, vocab context, quick action modes, and response
              language settings.
            </p>
          </div>
        </div>
      </section>

      <section id="stealth-layers">
        <h2 data-animate="fade-up">9 Layers of Stealth</h2>
        <p
          className="section-subtitle"
          data-animate="fade-up"
          data-delay="1"
        >
          Every detection vector covered. Every layer independently toggleable.
        </p>
        <div className="stealth-grid">
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="0"
          >
            <div className="stealth-status active" />
            <h3>Screen Capture Evasion</h3>
            <p>
              Window is invisible to all screen recording and screenshot tools
              via <code>setSharingType:0</code>.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="1"
          >
            <div className="stealth-status active" />
            <h3>Process Masquerading</h3>
            <p>
              Disguises its process name at runtime. Scans for known proctoring
              software on launch.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="1"
          >
            <div className="stealth-status active" />
            <h3>Window Focus Evasion</h3>
            <p>
              Elevated window level, excluded from Expose/Spaces. Passthrough
              mode prevents focus-steal detection.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="2"
          >
            <div className="stealth-status active" />
            <h3>Network Traffic Stealth</h3>
            <p>
              Spoofed User-Agent, request jitter, and optional proxy support to
              mask API traffic patterns.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="2"
          >
            <div className="stealth-status active" />
            <h3>Clipboard Bypass</h3>
            <p>
              Types text directly via CGEvents or uses ephemeral clipboard writes
              that clear in under 50ms.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="3"
          >
            <div className="stealth-status active" />
            <h3>Multi-Display Awareness</h3>
            <p>
              Detects all connected displays, virtual screens, and mirror mode.
              Auto-positions on secondary monitor.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="3"
          >
            <div className="stealth-status active" />
            <h3>Proctoring Detection</h3>
            <p>
              Scans running processes, localhost ports, and Launch Agents/Daemons
              for known proctoring tools.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="4"
          >
            <div className="stealth-status active" />
            <h3>Environment Report</h3>
            <p>
              Detects VMs, checks MAC OUIs, scans for VM kexts, and generates a
              stealth score from 0-100.
            </p>
          </div>
          <div
            className="stealth-card"
            data-animate="scale-in"
            data-delay="4"
          >
            <div className="stealth-status active" />
            <h3>Dodge on Hover</h3>
            <p>
              Auto-moves the window to the opposite corner after 2 seconds of
              cursor hover, with smooth animation.
            </p>
          </div>
        </div>
      </section>

      <section id="how-it-works">
        <h2 data-animate="fade-up">How it works</h2>
        <div className="steps">
          <div className="step" data-animate="slide-left" data-delay="0">
            <span className="step-number">1</span>
            <h3>Capture</h3>
            <p>
              Press <kbd>&#8984;&#8679;S</kbd> — Phantom takes a screenshot
              using native macOS APIs. The app window is excluded automatically.
            </p>
          </div>
          <div className="step" data-animate="slide-left" data-delay="1">
            <span className="step-number">2</span>
            <h3>Analyze</h3>
            <p>
              The screenshot is sent to Google Gemini with your chosen prompt.
              Processing takes just a few seconds.
            </p>
          </div>
          <div className="step" data-animate="slide-left" data-delay="2">
            <span className="step-number">3</span>
            <h3>Read</h3>
            <p>
              The AI response appears in a floating overlay at the corner of
              your screen — always on top, invisible to others.
            </p>
          </div>
        </div>
      </section>

      <section id="shortcuts">
        <h2 data-animate="fade-up">Shortcuts</h2>
        <div className="shortcuts-list">
          <div className="shortcut-item" data-animate="flip-in" data-delay="0">
            <kbd>&#8984;&#8679;S</kbd>
            <span>Capture &amp; Analyze</span>
          </div>
          <div className="shortcut-item" data-animate="flip-in" data-delay="1">
            <kbd>&#8984;&#8679;M</kbd>
            <span>Toggle Recording</span>
          </div>
          <div className="shortcut-item" data-animate="flip-in" data-delay="2">
            <kbd>&#8984;&#8679;A</kbd>
            <span>Toggle Response Panel</span>
          </div>
          <div className="shortcut-item" data-animate="flip-in" data-delay="3">
            <kbd>&#8984;&#8679;C</kbd>
            <span>Toggle Settings</span>
          </div>
        </div>
      </section>

      <section id="download">
        <h2 data-animate="fade-up">Get Phantom</h2>
        <p data-animate="fade-up" data-delay="1">
          Free and open source. Download, add your Gemini API key, and go.
        </p>
        <a
          href="/downloads/Phantom.dmg"
          className="btn-download"
          data-animate="scale-in"
          data-delay="2"
        >
          Download for macOS
        </a>
        <span className="hero-note" data-animate="fade-up" data-delay="3">
          macOS 12+ &middot; Apple Silicon &middot; v0.1.0
        </span>
      </section>

      <footer>
        <p>
          Built with{" "}
          <a href="https://tauri.app/" target="_blank" rel="noopener">
            Tauri
          </a>{" "}
          and{" "}
          <a href="https://ai.google.dev/" target="_blank" rel="noopener">
            Google Gemini
          </a>
        </p>
        <p>
          <a
            href="https://github.com/Enzo3322/phantom-ai"
            target="_blank"
            rel="noopener"
          >
            GitHub
          </a>
        </p>
      </footer>
    </>
  );
}
