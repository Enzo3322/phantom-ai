import { useRecording } from "./useRecording";
import { useTranscript } from "./useTranscript";

export function useTranscription() {
  const recording = useRecording();
  const transcript = useTranscript();
  return { ...recording, ...transcript };
}
