// Put testable mod logic here. This file is both the standalone Cargo library
// and the injected `crate::game::create_horizon` module; no generated copy exists.

pub fn block_break_message(pos: [i32; 3]) -> String {
    format!(
        "[create_horizon] Breaking block at {}, {}, {}",
        pos[0], pos[1], pos[2]
    )
}

pub fn before_break(pos: [i32; 3]) {
    eprintln!("{}", block_break_message(pos));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_preserves_signed_block_coordinates() {
        assert_eq!(
            block_break_message([-3, 64, 12]),
            "[create_horizon] Breaking block at -3, 64, 12"
        );
    }
}
