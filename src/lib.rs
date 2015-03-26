#![feature(trace_macros)]

#[macro_export]
macro_rules! machine (
  ( $machine:ident {
      attributes {
        $($name:ident : $t:ty),*
      }

      states {
        $($states:tt)*
      }
    }
  ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $machine<T> {
      state: T,
      $($name : $t),*
    }

    state_impl!($machine, { $($name),* }, $($states)* );
  )
);

#[macro_export]
macro_rules! state_impl (
  ($machine:ident, {$($field : ident),*}, $state:ident {  } => { $($transitions:tt)* }; $($rest:tt)* ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state;

    impl $machine<$state> {
      transitions_impl!($machine, $state, {$($field),*}, $($transitions)*);
    }

    state_impl!($machine, {$($field),*}, $($rest)*);
  );

  ($machine:ident, {$($field : ident),*}, $state:ident { $($name:ident : $t:ty),* } => { $($transitions:tt)* }; $($rest:tt)*) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state { $($name : $t,)* }

    impl $machine<$state> {
      transitions_impl!($machine, $state, {$($field),*}, $($transitions)*);
    }

    state_impl!($machine, {$($field),*}, $($rest)*);
  );

  ($machine:ident, {$($field : ident),*}, $state:ident {  }; $($rest:tt)* ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state;

    state_impl!($machine, {$($field),*}, $($rest)*);
  );

  ($machine:ident, {$($field : ident),*}, $state:ident { $($name:ident : $t:ty),* }; $($rest:tt)*) => (
    #[erive(PartialEq,Eq,Debug)]
    pub struct $state { $($name : $t,)* }

    state_impl!($machine, {$($field),*}, $($rest)*);
  );

  ($machine:ident, {$($field : ident),*}, ) =>  ( );
);

#[macro_export]
macro_rules! transitions_impl (
  ($machine:ident, $state:ident, {$($field : ident),*}, ) => (
  );

  ($machine:ident, $state:ident, {$($field : ident),*}, $name:ident ( $( $arg:ident : $t:ty ),* ) => $next:ident $b:block; $($rest:tt)* ) => (
    trans_impl!($machine, $state, {$($field),*}, $name ( $( $arg : $t ),* ) => $next $b;);
    transitions_impl!($machine, $state, {$($field),*}, $($rest)*);
  );
  ($machine:ident, $state:ident, {$($field : ident),*}, $name:ident => $next:ident; $($rest:tt)* ) => (

    trans_impl!($machine, $state, {$($field),*}, $name => $next;);
    transitions_impl!($machine, $state, {$($field),*}, $($rest)*);
  );
  ($machine:ident, $state:ident, {$($field : ident),*}, $name:ident => $next:ident $b:block; $($rest:tt)* ) => (

    trans_impl!($machine, $state, {$($field),*}, $name  => $next $b;);
    transitions_impl!($machine, $state, {$($field),*}, $($rest)*);
  );

);

#[macro_export]
macro_rules! trans_impl (
  ($machine:ident, $state:ident, {$($field : ident),*}, $name:ident => $next:ident;) => (
    pub fn $name(&self) -> $machine<$next> {
      return $machine {
        state: $next,
        $($field : self.$field),*
      }
    }
  );
  ($machine:ident, $state:ident, {$($field : ident),*}, $name:ident ( $( $arg:ident : $t:ty ),* ) => $next:ident $b:block; ) => (
    pub fn $name(&self,  $( $arg : $t ),* ) -> $machine<$next> {
      let res = $b;
      return $machine {
        state: res,
        $($field : self.$field),*
      }
    }
  );
  ($machine:ident, $state:ident, {$($field : ident),*}, $name:ident => $next:ident $b:block;) => (
    pub fn $name(&self) -> $machine<$next> {
      let res = $b;
      return $machine {
        state: res,
        $($field : self.$field),*
      }
    }
  );
);

#[cfg(test)]
mod tests {
  #![feature(trace_macros)]
  use super::*;

  trace_macros!(true);
  machine!(Machine {
    attributes {
      counter: u8
    }

    states {

      SA { } => {
        tr1(a:u8) => SB
        {
          println!("current: {}", a);
          SB { b: a, c: false }
        };

      };

      SB {
        b: u8,
        c: bool
      } => {
        tr2 => SC;
        tr3 => SA {
          SA
        };
      };

      SC {

      } => {
        tr1 => SA;
      };

    }
  });
  trace_macros!(false);

  #[test]
  fn transitions() {
    let m = Machine { state: SB { b: 1, c: true }, counter: 0 };
    let m = m.tr3();
    assert_eq!(m, Machine { state: SA, counter: 0} );
    let m = m.tr1(42);
    assert_eq!(m, Machine { state: SB { b:42, c:false } , counter: 0 } );
    let m = m.tr2().tr1();
    assert_eq!(m, Machine { state: SA , counter: 0 } );
  }
}
