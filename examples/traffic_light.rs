#[macro_use]
extern crate machine;

machine!(
  #[derive(Clone,Debug,PartialEq)]
  enum Traffic {
    Green { count: u8 },
    Orange,
    Red,
  }
);

#[derive(Clone,Debug,PartialEq)]
pub struct Advance;

#[derive(Clone,Debug,PartialEq)]
pub struct PassCar { count: u8 }

transitions!(Traffic,
  [
    (Green, Advance) => Orange,
    (Orange, Advance) => Red,
    (Red, Advance) => Green,
    (Green, PassCar) => [Green, Orange,],
  ]
);

impl Green {
  pub fn on_advance(self, _: Advance) -> Orange {
    Orange {}
  }

  pub fn on_pass_car(self, input: PassCar) -> Traffic {
    let count = self.count + input.count;
    if count >= 10 {
      println!("reached max cars count: {}", count);
      Traffic::orange()
    } else {
      Traffic::green(count)
    }
  }

}

impl Orange {
  pub fn on_advance(self, _: Advance) -> Red {
    Red {}
  }
}

impl Red {
  pub fn on_advance(self, _: Advance) -> Green {
    Green {
      count: 0
    }
  }
}

methods!(Traffic,
  [
    Green => get count: u8,
    Green => set count: u8,
    [Green, Orange, Red,] => fn can_pass(&self) -> bool
  ]
);

impl Green {
  pub fn can_pass(&self) -> bool {
    true
  }
}

impl Orange {
  pub fn can_pass(&self) -> bool {
    false
  }
}

impl Red {
  pub fn can_pass(&self) -> bool {
    false
  }
}

fn main() {
  let mut t = Traffic::Green(Green { count: 0 });
  t = t.on_pass_car(PassCar { count: 1});
  t = t.on_pass_car(PassCar { count: 2});
  assert_eq!(t, Traffic::green(3));
  t = t.on_advance(Advance);
  assert_eq!(t, Traffic::orange());

  t = t.on_advance(Advance);
  assert_eq!(t, Traffic::red());

  t = t.on_advance(Advance);
  assert_eq!(t, Traffic::green(0));
  t = t.on_pass_car(PassCar { count: 5 });
  assert_eq!(t, Traffic::green(3));
  t = t.on_pass_car(PassCar { count: 7 });
  assert_eq!(t, Traffic::orange());
  t = t.on_advance(Advance);
  assert_eq!(t, Traffic::red());
  t = t.on_pass_car(PassCar { count: 7 });
  assert_eq!(t, Traffic::error());
  t = t.on_advance(Advance);
  assert_eq!(t, Traffic::error());
}
