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


#[derive(PartialEq,Eq,Debug)]
pub struct SA;
#[derive(PartialEq,Eq,Debug)]
pub struct SB { b: u8 }
#[derive(PartialEq,Eq,Debug)]
pub struct SC;

#[derive(PartialEq,Eq,Debug)]
struct Machine<T> {
  state: T,
  counter: u8
}

impl<T> Machine<T> {
  pub fn cnt(&self) -> u8 {
    self.counter
  }
}

impl Machine<SA> {
  fn tr1(&self) -> Machine<SB> {
    println!("go to SB");
    return Machine {
      state: SB { b: 1 },
      counter: self.counter
    }
  }
}

impl Machine<SB> {
  fn tr2(&self) -> Machine<SC> {
    println!("go to SC");
    return Machine {
      state: SC,
      counter: 0
    }
  }

  fn tr3(&self) -> Machine<SA> {
    println!("go to SA");
    return Machine {
      state: SA,
      counter: 0
    }
  }
}

impl Machine<SC> {
  fn tr4(&self) -> Machine<SA> {
    println!("go to SA");
    return Machine {
      state: SA,
      counter: 0
    }
  }
}

fn main() {
  let m = Machine { state: SA, counter: 1 };
  let m = m.tr1();
  let m = m.tr2();
  let m = m.tr4();

  println!("counter: {}", m.cnt());
/*
  let m = Machine { state: SA };
  let m = m.tr1();
  let m = m.tr4();
*/
}
