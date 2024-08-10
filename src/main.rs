use async_read_window::AsyncReadWindow;
use futures_lite::AsyncReadExt;
use smol::block_on;
use smol::fs::File;
use smol::io::Cursor;

mod async_read_window;

fn main() {
    block_on(async { file().await })
}

async fn file() {
    let file = File::open("file.txt").await.unwrap();

    let mut read_window = AsyncReadWindow::new(file, 7, 6).await.unwrap();

    let mut buffer = vec![];
    println!("start read");
    read_window.read_to_end(&mut buffer).await.unwrap();
    println!("end read");

    assert_eq!(buffer.as_slice(), b"part 2");
}

async fn buffer() {
    let buffer = include_bytes!("../file.txt");

    let cursor = Cursor::new(buffer);

    let mut read_window = AsyncReadWindow::new(cursor, 7, 6).await.unwrap();

    let mut read_buffer = vec![];
    read_window.read_to_end(&mut read_buffer).await.unwrap();

    assert_eq!(read_buffer.as_slice(), b"part 2");
}
