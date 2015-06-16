/// State A    => {
///   tr1 => State B(x),
/// }
///
/// State B(x) => {
///   tr2 => State C
///   tr3 => State A
/// }
///
/// State C    => {
///   tr4 => State A
/// }
///
/// generates following interface

/*
#[derive(PartialEq,Eq,Debug)]
pub struct SA;
#[derive(PartialEq,Eq,Debug)]
pub struct SB { b: u8 }
#[derive(PartialEq,Eq,Debug)]
pub struct SC;
*/

#[derive(PartialEq,Eq,Debug)]
pub enum State {
  SA,
  SB(u8),
  SC,
  Error
}

#[derive(PartialEq,Eq,Debug)]
struct Machine {
  state: State,
  counter: u8
}

impl Machine {
  pub fn cnt(&self) -> u8 {
    self.counter
  }

  fn tr1(&mut self) -> Option<()> {
    match self.state {
      State::SA => {
        println!("go to SB");
        self.state = State::SB(1);
        Some(())
      },
      _ => {
        self.state = State::Error;
        None
      }
    }
  }

  fn tr2(&mut self) -> Option<()> {
    match self.state {
      State::SB(i) => {
        println!("go to SC");
        self.state = State::SC;
        Some(())
      },
      _ => {
        self.state = State::Error;
        None
      }
    }
  }

  fn tr3(&mut self) -> Option<()> {
    match self.state {
      State::SB(i) => {
        println!("go to SA");
        self.state = State::SA;
        Some(())
      },
      _ => {
        self.state = State::Error;
        None
      }
    }
  }

  fn tr4(&mut self) -> Option<()> {
    match self.state {
      State::SC => {
        println!("go to SA");
        self.state = State::SA;
        Some(())
      },
      _ => {
        self.state = State::Error;
        None
      }
    }
  }
}

fn main() {
  let mut m = Machine { state: State::SA, counter: 1 };
  println!("machine: {:?}", m);
  m.tr1();
  println!("machine: {:?}", m);
  m.tr2();
  println!("machine: {:?}", m);
  m.tr4();
  println!("machine: {:?}", m);

  println!("counter: {}", m.cnt());
/*
  let m = Machine { state: SA };
  let m = m.tr1();
  let m = m.tr4();
*/
}
