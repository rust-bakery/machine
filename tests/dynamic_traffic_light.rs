#[macro_use]
extern crate machine;

machine!(
  enum TrafficLight {
    Green { count: u8 },
    Orange,
    Red,
    BlinkingOrange,
  }
);

#[derive(Clone,Debug,PartialEq)]
pub struct Advance;

#[derive(Clone,Debug,PartialEq)]
pub struct PassCar { count: u8 }

transitions!(TrafficLight,
  [
    (Green, Advance) => Orange,
    (Orange, Advance) => Red,
    (Red, Advance) => Green,
    (Green, PassCar) => [Green, Orange]
  ]
);

impl Green {
  pub fn on_Advance(self, _: Advance) -> Orange {
    Orange {}
  }

  pub fn on_PassCar(self, input: PassCar) -> TrafficLight {
    let count = self.count + input.count;
    if count >= 10 {
      println!("reached max cars count: {}", count);
      TrafficLight::orange()
    } else {
      TrafficLight::green(count)
    }
  }
}

impl Orange {
  pub fn on_Advance(self, _: Advance) -> Red {
    Red {}
  }
}

impl Red {
  pub fn on_Advance(self, _: Advance) -> Green {
    Green {
      count: 0
    }
  }
}

#[test]
fn test() {
  let mut t = TrafficLight::Green(Green { count: 0 });
  t = t.on_PassCar(PassCar { count: 1}).unwrap();
  t = t.on_PassCar(PassCar { count: 2}).unwrap();
  assert_eq!(t, TrafficLight::green(3));
  t = t.on_Advance(Advance).unwrap();
  //println!("trace: {}", t.print_trace());
  assert_eq!(t, TrafficLight::orange());

  t = t.on_Advance(Advance).unwrap();
  assert_eq!(t, TrafficLight::red());

  t = t.on_Advance(Advance).unwrap();
  assert_eq!(t, TrafficLight::green(0));
  t = t.on_PassCar(PassCar { count: 5 }).unwrap();
  assert_eq!(t, TrafficLight::green(5));
  t = t.on_PassCar(PassCar { count: 7 }).unwrap();
  assert_eq!(t, TrafficLight::orange());
  t = t.on_Advance(Advance).unwrap();
  assert_eq!(t, TrafficLight::red());
}
