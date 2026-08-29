# Per-series code snippets. Edit this, then: python3 cards.py  (reads cards_data.py)
# Each: name -> (title, code, line_numbers). Gate comments with ①② -> amber(writer), ③④ -> teal(reader).
CARDS = {
  "example": ("file.rs — what it shows",
'''pub fn demo(&self) {
    self.state.fetch_add(1, Relaxed);   // ① writer gate note (amber)
    let x = self.state.load(Acquire);   // ③ reader gate note (teal)
}''', True),
}
