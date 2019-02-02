#[macro_use]
extern crate machine;

pub trait Transitions {
  fn next(&mut self);
}

machine!(
  #[derive(Clone,Debug,PartialEq)]
  enum State {
    Start { pub x:u8 },
    End { pub x: u8, y: bool },
  }
);

#[derive(Clone,Debug,PartialEq)]
pub struct Msg1;

#[derive(Clone,Debug,PartialEq)]
pub struct Msg2;

transitions!(State,
  [
  (Start, Msg1) => End,
  (End, Msg1) => End
  ]
);

impl Start {
  pub fn on_msg1(self, _input: Msg1) -> End {
    End {
      x: self.x,
      y: true,
    }
  }
}

impl End {
  pub fn on_msg1(self, _input: Msg1) -> End {
    End {
      x: self.x,
      y: !self.y,
    }
  }
}

#[test]
fn hello() {
  let start = State::start(0);
  let _end  = State::end(1, true);
  let _err = State::error();

  assert_eq!(start, State::Start(Start { x: 0 }));
}
