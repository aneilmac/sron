use core::task::{Context, Poll};
use futures::io::IoSlice;
use hyper::client::connect::{Connected, Connection};
use std::io::Error;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Hole implements the required, [Connection], [AsyncRead] and [AsyncWrite] traits
/// required for a Service response, but throws away all data and returns EOF if
/// read. Internally, this uses [tokio::io::Sink] and [tokio::io::Empty] to throw
/// away writes and reads respectively.
pub struct Hole {
    in_data: tokio::io::Sink,
    out_data: tokio::io::Empty,
}

impl Hole {
    pub fn new() -> Hole {
        Hole {
            in_data: tokio::io::sink(),
            out_data: tokio::io::empty(),
        }
    }
}

impl Hole {
    fn get_in(self: Pin<&mut Self>) -> Pin<&mut tokio::io::Sink> {
        // This is okay because `field` is pinned when `self` is.
        unsafe { self.map_unchecked_mut(|s| &mut s.in_data) }
    }

    fn get_out(self: Pin<&mut Self>) -> Pin<&mut tokio::io::Empty> {
        // This is okay because `field` is pinned when `self` is.
        unsafe { self.map_unchecked_mut(|s| &mut s.out_data) }
    }
}

impl AsyncWrite for Hole {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.get_in().poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.get_in().poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.get_in().poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, Error>> {
        self.get_in().poll_write_vectored(cx, bufs)
    }
    fn is_write_vectored(&self) -> bool {
        self.in_data.is_write_vectored()
    }
}

impl AsyncRead for Hole {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.get_out().poll_read(cx, buf)
    }
}

impl Connection for Hole {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}
