#[macro_use]
extern crate machine;

machine!(
  #[derive(Clone,Debug,PartialEq)]
  enum TrafficLight {
    Green { count: u8 },
    Orange,
    Red,
    BlinkingOrange,
  }
);

pub mod prefix {
  #[derive(Clone,Debug,PartialEq)]
  pub struct Advance;
}

#[derive(Clone,Debug,PartialEq)]
pub struct PassCar<'a, T> { count: u8, name: &'a T }

#[derive(Clone,Debug,PartialEq)]
pub struct Toggle;

transitions!(TrafficLight,
  [
    (Green, prefix::Advance) => Orange,
    (Orange, prefix::Advance) => Red,
    (Red, prefix::Advance) => Green,
    (Green, PassCar<'a, T>) => [Green, Orange],
    (Green, Toggle) => BlinkingOrange,
    (Orange, Toggle) => BlinkingOrange,
    (Red, Toggle) => BlinkingOrange,
    (BlinkingOrange, Toggle) => Red
  ]
);

methods!(TrafficLight,
  [
    Green => get count: u8,
    Green => set count: u8,
    Green, Orange, Red, BlinkingOrange => default(false) fn working(&self) -> bool
  ]
);

impl Green {
  pub fn on_advance(self, _: prefix::Advance) -> Orange {
    Orange {}
  }

  pub fn on_pass_car<'a, T>(self, input: PassCar<'a, T>) -> TrafficLight {
    let count = self.count + input.count;
    if count >= 10 {
      println!("reached max cars count: {}", count);
      TrafficLight::orange()
    } else {
      TrafficLight::green(count)
    }
  }

  pub fn on_toggle(self, _: Toggle) -> BlinkingOrange {
    BlinkingOrange{}
  }

  pub fn working(&self) -> bool {
    true
  }
}

impl Orange {
  pub fn on_advance(self, _: prefix::Advance) -> Red {
    Red {}
  }

  pub fn on_toggle(self, _: Toggle) -> BlinkingOrange {
    BlinkingOrange{}
  }

  pub fn working(&self) -> bool {
    true
  }
}

impl Red {
  pub fn on_advance(self, _: prefix::Advance) -> Green {
    Green {
      count: 0
    }
  }

  pub fn on_toggle(self, _: Toggle) -> BlinkingOrange {
    BlinkingOrange{}
  }

  pub fn working(&self) -> bool {
    true
  }
}

impl BlinkingOrange {
  pub fn on_toggle(self, _: Toggle) -> Red {
    Red{}
  }

  pub fn working(&self) -> bool {
    false
  }
}

#[test]
fn test() {
  use prefix::Advance;

  let mut t = TrafficLight::Green(Green { count: 0 });
  t = t.on_pass_car(PassCar { count: 1, name: &"test".to_string() });
  t = t.on_pass_car(PassCar { count: 2, name: &"test".to_string() });
  assert_eq!(t, TrafficLight::green(3));
  t = t.on_advance(Advance);
  //println!("trace: {}", t.print_trace());
  assert_eq!(t, TrafficLight::orange());

  t = t.on_advance(Advance);
  assert_eq!(t, TrafficLight::red());

  t = t.on_advance(Advance);
  assert_eq!(t, TrafficLight::green(0));
  t = t.on_pass_car(PassCar { count: 5, name: &"test".to_string() });
  assert_eq!(t, TrafficLight::green(5));
  t = t.on_pass_car(PassCar { count: 7, name: &"test".to_string() });
  assert_eq!(t, TrafficLight::orange());
  t = t.on_advance(Advance);
  assert_eq!(t, TrafficLight::red());
  t = t.on_pass_car(PassCar { count: 7, name: &"test".to_string() });
  assert_eq!(t, TrafficLight::error());
  t = t.on_advance(Advance);
  assert_eq!(t, TrafficLight::error());
}
