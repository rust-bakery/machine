/// State A    => {
///   tr1 => State B(x),
///   tr4 => State A
/// }
///
/// State B(x) => {
///   tr2 => State C
///   tr3 => State A
///   tr5 => State A -> u8
/// }
///
/// State C    => {
///   tr4     => State A
///   tr6(u8) => State B
/// }
///
/// generates following interface

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
        println!("{} =>go to SC", i);
        self.state = State::SC;
        Some(())
      },
      _ => {
        self.state = State::Error;
        None
      }
    }
  }

  #[allow(dead_code)]
  fn tr3(&mut self) -> Option<()> {
    match self.state {
      State::SB(i) => {
        println!("{} => go to SA", i);
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
      State::SA => {
        println!("stay in SA");
        self.state = State::SA;
        Some(())
      },
      _ => {
        self.state = State::Error;
        None
      }
    }
  }

  fn tr5(&mut self) -> Option<u8> {
    match self.state {
      State::SB(i) => {
        println!("go to SA");
        self.state = State::SA;
        Some(i)
      },
      _ => {
        self.state = State::Error;
        None
      }
    }
  }

  fn tr6(&mut self, i: u8) -> Option<()> {
    match self.state {
      State::SC => {
        println!("go to SB");
        self.state = State::SB(i);
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
  m.tr6(42);
  println!("machine: {:?}", m);
  let a = m.tr5();
  println!("machine: {:?} -> {:?}", m, a);
  m.tr4();
  println!("machine: {:?}", m);

  println!("counter: {}", m.cnt());
/*
  let m = Machine { state: SA };
  let m = m.tr1();
  let m = m.tr4();
*/
}
