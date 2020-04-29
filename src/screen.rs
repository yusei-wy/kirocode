pub struct Screen {
    pub rows: usize,
    pub cols: usize,
}

impl Screen {
    pub fn new(size: Option<(usize, usize)>) -> Screen {
        let (w, h) = if let Some(s) = size {
            s
        } else {
            get_window_size()
        };

        Screen { rows: w, cols: h }
    }
}

fn get_window_size() -> (usize, usize) {
    if let Some(s) = term_size::dimensions_stdout() {
        return s;
    };

    // TODO: window size を取得できなかったのときのフォールバック
    return (0, 0);
}
