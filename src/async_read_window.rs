use std::error::Error;
use std::{cmp::min, pin::Pin, task::Poll};

use futures_lite::io::{AsyncRead, AsyncSeek, ErrorKind, SeekFrom};
use futures_lite::{ready, AsyncSeekExt};

pub struct AsyncReadWindow<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    start: u64,
    size: u64,
    reader: R,
}

impl<R> AsyncReadWindow<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    pub async fn new(mut reader: R, start: u64, size: u64) -> Result<Self, Box<dyn Error>> {
        reader.seek(SeekFrom::Start(start)).await?;
        Ok(Self {
            reader,
            start,
            size,
        })
    }
}

impl<R> AsyncRead for AsyncReadWindow<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<futures_lite::io::Result<usize>> {
        let this = self.get_mut();

        let mut reader = Pin::new(&mut this.reader);

        let mut stream_position = ready!(reader.as_mut().poll_seek(cx, SeekFrom::Current(0)))?;

        if stream_position < this.start {
            stream_position = ready!(reader.as_mut().poll_seek(cx, SeekFrom::Start(this.start)))?;
        }

        let offset_from_start = stream_position - this.start;
        if offset_from_start > this.size {
            let end_offset = this.start + this.size;
            stream_position = ready!(reader.as_mut().poll_seek(cx, SeekFrom::Start(end_offset)))?;
        }

        let offset_from_start = stream_position - this.start;
        let max_read_length = (this.size - offset_from_start) as usize;

        let read_length = min(buf.len(), max_read_length);
        let poll = reader.as_mut().poll_read(cx, &mut buf[..read_length]);

        if let Poll::Ready(ref result) = poll {
            println!("{:#?}", result);
        } else {
            println!("pending");
        }
        poll
    }
}

impl<R> AsyncSeek for AsyncReadWindow<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        pos: SeekFrom,
    ) -> std::task::Poll<futures_lite::io::Result<u64>> {
        let this = self.get_mut();
        let mut reader = Pin::new(&mut this.reader);
        match pos {
            SeekFrom::Start(pos) => {
                let seek_pos = this.start + pos;
                reader.poll_seek(cx, SeekFrom::Start(seek_pos))
            }
            SeekFrom::End(pos) => {
                let end_pos = this.start + this.size;

                let seek_pos = end_pos as i128 + pos as i128;

                if seek_pos < this.start as i128 {
                    return Poll::Ready(Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Tried to seek beyond the start of the file.",
                    )));
                }

                reader.poll_seek(cx, SeekFrom::Start(seek_pos as u64))
            }
            SeekFrom::Current(pos) => {
                let current_pos = ready!(reader.as_mut().poll_seek(cx, SeekFrom::Current(0)))?;

                let seek_pos = current_pos as i128 + pos as i128;

                if seek_pos < this.start as i128 {
                    return Poll::Ready(Err(std::io::Error::new(
                        ErrorKind::InvalidInput,
                        "Tried to seek beyond the start of the file.",
                    )));
                }

                reader.poll_seek(cx, SeekFrom::Current(pos))
            }
        }
    }
}
