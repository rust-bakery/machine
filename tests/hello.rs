#[macro_use]
extern crate machine;

pub trait Transitions {
  fn next(&mut self);
}

machine!(
  enum State {
    Start { pub x:u8 },
    End { pub x: u8, y: bool },
    Error,
  }
);

#[derive(Clone,Debug,PartialEq)]
pub struct Msg1;

#[derive(Clone,Debug,PartialEq)]
pub struct Msg2;

transitions!(State,
  [
  (Start, Msg1) => End,
  (End, Msg1) => End,
  (Start, Msg2) => Error
  ]
);

impl Start {
  pub fn on_Msg1(self, input: Msg1) -> End {
    End {
      x: self.x,
      y: true,
    }
  }

  pub fn on_Msg2(self, input: Msg2) -> Error {
    Error {}
  }
}

impl End {
  pub fn on_Msg1(self, input: Msg1) -> End {
    End {
      x: self.x,
      y: !self.y,
    }
  }
}

#[test]
fn hello() {
  let start = State::start(0);
  let end  = State::end(1, true);
  let err = State::error();

  assert_eq!(start, State::Start(Start { x: 0 }));
}
