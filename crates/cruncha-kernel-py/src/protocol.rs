// SPDX-License-Identifier: GPL-3.0-or-later
//! Framing, the same as `python/cruncha_worker/protocol.py`: a 4-byte big-endian length and a
//! UTF-8 JSON header; a header with `"blobs": n` is followed by `n` raw byte frames.

use serde_json::Value as Json;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Frames are capped well below 4 GiB.
pub const MAX_FRAME: u32 = 1 << 31;

async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<Vec<u8>> {
    let n = r.read_u32().await?;
    if n >= MAX_FRAME {
        return Err(std::io::Error::other(format!("frame too large: {n}")));
    }
    let mut buf = vec![0; n as usize];
    r.read_exact(&mut buf).await?;
    Ok(buf)
}

pub async fn read_message<R: AsyncRead + Unpin>(
    r: &mut R,
) -> std::io::Result<(Json, Vec<Vec<u8>>)> {
    let header: Json =
        serde_json::from_slice(&read_frame(r).await?).map_err(std::io::Error::other)?;
    let n = header.get("blobs").and_then(Json::as_u64).unwrap_or(0);
    let mut blobs = Vec::with_capacity(n as usize);
    for _ in 0..n {
        blobs.push(read_frame(r).await?);
    }
    Ok((header, blobs))
}

pub async fn write_message<W: AsyncWrite + Unpin>(
    w: &mut W,
    header: &Json,
    blobs: &[Vec<u8>],
) -> std::io::Result<()> {
    let mut header = header.clone();
    if !blobs.is_empty() {
        header["blobs"] = blobs.len().into();
    }
    let data = serde_json::to_vec(&header).map_err(std::io::Error::other)?;
    let len = |n: usize| {
        u32::try_from(n)
            .ok()
            .filter(|n| *n < MAX_FRAME)
            .ok_or_else(|| std::io::Error::other("frame too large"))
    };
    w.write_u32(len(data.len())?).await?;
    w.write_all(&data).await?;
    for b in blobs {
        w.write_u32(len(b.len())?).await?;
        w.write_all(b).await?;
    }
    w.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trip() {
        let mut buf = Vec::new();
        write_message(
            &mut buf,
            &serde_json::json!({"op": "x"}),
            &[b"abc".to_vec(), vec![]],
        )
        .await
        .unwrap();
        let (h, blobs) = read_message(&mut buf.as_slice()).await.unwrap();
        assert_eq!(h["op"], "x");
        assert_eq!(h["blobs"], 2);
        assert_eq!(blobs, vec![b"abc".to_vec(), vec![]]);
    }
}
