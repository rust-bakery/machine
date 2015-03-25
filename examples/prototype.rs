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
  state: T
}

impl Machine<SA> {
  fn tr1(&self) -> Machine<SB> {
    println!("go to SB");
    return Machine {
      state: SB { b: 1 }
    }
  }
}

impl Machine<SB> {
  fn tr2(&self) -> Machine<SC> {
    println!("go to SC");
    return Machine {
      state: SC
    }
  }

  fn tr3(&self) -> Machine<SA> {
    println!("go to SA");
    return Machine {
      state: SA
    }
  }
}

impl Machine<SC> {
  fn tr4(&self) -> Machine<SA> {
    println!("go to SA");
    return Machine {
      state: SA
    }
  }
}

fn main() {
  let m = Machine { state: SA };
  let m = m.tr1();
  let m = m.tr2();
  let m = m.tr4();
/*
  let m = Machine { state: SA };
  let m = m.tr1();
  let m = m.tr4();
*/
}
