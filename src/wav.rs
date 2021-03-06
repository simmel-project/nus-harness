extern crate byteorder;
use std;
use std::io::prelude::*;
use std::fs::File;
use self::byteorder::{LittleEndian, WriteBytesExt};

const FORMAT_PCM  : u16 = 1;

pub fn write_wav(rate: u32, samples: &[i16], file: &mut File) -> std::io::Result<()> {
    let bits_per_sample = 16;
    let num_channels = 1;
    /* chunkId */       file.write_all(&[0x52, 0x49, 0x46, 0x46])?;        // 'RIFF'
    /* chunkSize */     file.write_u32::<LittleEndian>(36 + (samples.len() as u32 * (bits_per_sample / 8)))?;
    /* format */        file.write_all(&[0x57, 0x41, 0x56, 0x45])?;        // 'WAVE'
    /* subChunk1Id */   file.write_all(&[0x66, 0x6d, 0x74, 0x20])?;        // 'fmt '
    /* subChunk1Size */ file.write_u32::<LittleEndian>(16 as u32)?;             // 16 bytes for PCM
    /* audioFormat */   file.write_u16::<LittleEndian>(FORMAT_PCM)?;            // 1 = PCM
    /* numChannels */   file.write_u16::<LittleEndian>(num_channels as u16)?;   // 1 = Mono
    /* sampleRate */    file.write_u32::<LittleEndian>(rate)?;                  // Probably 44100
    /* byteRate */      file.write_u32::<LittleEndian>(rate * num_channels * (bits_per_sample / 8) as u32)?;
    /* blockAlign */    file.write_u16::<LittleEndian>(num_channels as u16 * (bits_per_sample / 8) as u16)?;
    /* bitsPerSample */ file.write_u16::<LittleEndian>(bits_per_sample as u16)?;
    /* subChunk2Id */   file.write_all(&[0x64, 0x61, 0x74, 0x61])?;        // 'data'
    /* subChunk2Size */ file.write_u32::<LittleEndian>(samples.len() as u32 * (bits_per_sample / 8))?;
    if cfg!(target_endian = "big") {
        for sample in samples {
            file.write_i16::<LittleEndian>(*sample)?;
        }
    } else {
        use std::{slice, mem};
        let slice_u8: &[u8] = unsafe {
            slice::from_raw_parts(
                samples.as_ptr() as *const u8,
                samples.len() * mem::size_of::<u16>()
            )
        };
        file.write_all(slice_u8)?;
    }

    Ok(())
}

/*
      chunkId      : [0x52,0x49,0x46,0x46], // 0    4    "RIFF" = 0x52494646
      chunkSize    : 0,                     // 4    4    36+SubChunk2Size = 4+(8+SubChunk1Size)+(8+SubChunk2Size)
      format       : [0x57,0x41,0x56,0x45], // 8    4    "WAVE" = 0x57415645
      subChunk1Id  : [0x66,0x6d,0x74,0x20], // 12   4    "fmt " = 0x666d7420
      subChunk1Size: 16,                    // 16   4    16 for PCM
      audioFormat  : 1,                     // 20   2    PCM = 1
      numChannels  : options.channels,      // 22   2    Mono = 1, Stereo = 2...
      sampleRate   : options.rate,          // 24   4    8000, 44100...
      byteRate     : 0,                     // 28   4    SampleRate*NumChannels*BitsPerSample/8
      blockAlign   : 0,                     // 32   2    NumChannels*BitsPerSample/8
      bitsPerSample: options.depth,                     // 34   2    8 bits = 8, 16 bits = 16
      subChunk2Id  : [0x64,0x61,0x74,0x61], // 36   4    "data" = 0x64617461
      subChunk2Size: 0                      // 40   4    data size = NumSamples*NumChannels*BitsPerSample/8
*/
