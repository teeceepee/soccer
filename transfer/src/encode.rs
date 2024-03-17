use crate::byte_shift::ByteShifter;

use futures::ready;
use tokio::io::{AsyncRead, AsyncWrite};

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Encode<'a, R: ?Sized, W: ?Sized> {
    reader: &'a mut R,
    read_done: bool,
    writer: &'a mut W,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: Box<[u8]>,

    sugar: u8,
}

pub fn encode<'a, R, W>(reader: &'a mut R, writer: &'a mut W, sugar: u8) -> Encode<'a, R, W>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    Encode {
        reader,
        read_done: false,
        writer,
        amt: 0,
        pos: 0,
        cap: 0,
        buf: Box::new([0; 2048]),

        sugar,
    }
}


impl<R, W> Future for Encode<'_, R, W>
    where
        R: AsyncRead + Unpin + ?Sized,
        W: AsyncWrite + Unpin + ?Sized,
{
    type Output = io::Result<u64>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        loop {
            // If our buffer is empty, then we need to read some data to
            // continue.
            if self.pos == self.cap && !self.read_done {
                let me = &mut *self;
                // TODO
                // let n = ready!(Pin::new(&mut *me.reader).poll_read(cx, &mut me.buf))?;
                let n = 0;
                if n == 0 {
                    self.read_done = true;
                } else {
                    self.pos = 0;
                    self.cap = n;
                }
            }

            // If our buffer has some data, let's write it out!
            while self.pos < self.cap {
                let me = &mut *self;

                // 字节编码
                let shifter = ByteShifter::new(me.sugar);

                let slice = &mut me.buf[me.pos..me.cap];
                for b in slice.iter_mut() {
                    *b = shifter.encode(*b);
                }

                let i = ready!(Pin::new(&mut *me.writer).poll_write(cx, slice))?;
                if i == 0 {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "write zero byte into writer",
                    )));
                } else {
                    self.pos += i;
                    self.amt += i as u64;
                }
            }

            // If we've written all the data and we've seen EOF, flush out the
            // data and finish the transfer.
            if self.pos == self.cap && self.read_done {
                let me = &mut *self;
                ready!(Pin::new(&mut *me.writer).poll_flush(cx))?;
                return Poll::Ready(Ok(self.amt));
            }
        }
    }
}
