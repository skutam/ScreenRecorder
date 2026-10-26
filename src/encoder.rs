use windows_capture::frame::Frame;

// Windows
struct MediaFoundationEncoder;
// NVIDIA
struct NvencEncoder;
// AMD
struct AmfEncoder;
// INTEL
struct QuickSyncEncoder;

struct EncodedPacket;
struct EncoderError;

trait VideoEncoder {
    fn encode(&mut self, frame: &Frame) -> Result<Option<EncodedPacket>, EncoderError>;
    fn finish(&mut self) -> Result<Option<EncodedPacket>, EncoderError>;
}