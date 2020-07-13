use std::io::Write;

pub struct Screen<W: Write> {
    pub rows: usize,
    pub cols: usize,
    // TODO: private にしたい
    pub output: W,
}

impl<W> Screen<W>
where
    W: Write,
{
    pub fn new(size: Option<(usize, usize)>, output: W) -> Self {
        if let Some(s) = size {
            return Self {
                rows: s.0,
                cols: s.1,
                output,
            };
        }

        let size = get_window_size();
        Self {
            rows: size.0,
            cols: size.1,
            output,
        }
    }
}

fn get_window_size() -> (usize, usize) {
    if let Some((w, h)) = term_size::dimensions() {
        return (w, h);
    }

    (0, 0)
}
