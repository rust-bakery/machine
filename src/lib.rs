#![feature(trace_macros)]

#[macro_export]
macro_rules! machine (
  ( $machine:ident {
      $($rest:tt)*
      //$($state_def:tt)*
      /*$($state:ident {
        //$($args:tt)*
        $args:tt
      };)*
      */
    }
  ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $machine<T> {
      state: T
    }

    state_impl!($machine, $($rest)* );
    //$(
    //state_impl!($machine, $state_def);
    //state_impl!($machine, $state { $args } );
    //)*
    /*$(
       #[derive(PartialEq,Eq,Debug)]
       pub struct $state { $($args)* };
    )* */
  )
);

#[macro_export]
macro_rules! state_impl (
  ($machine:ident, $state:ident {  } => { $($transitions:tt)* }; $($rest:tt)* ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state;

    impl $machine<$state> {
      transitions_impl!($machine, $state, $($transitions)*);
    }

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident, $state:ident { $($name:ident : $t:ty),* } => { $($transitions:tt)* }; $($rest:tt)*) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state { $($name : $t,)* }

    impl $machine<$state> {
      transitions_impl!($machine, $state, $($transitions)*);
    }

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident, $state:ident {  }; $($rest:tt)* ) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state;

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident, $state:ident { $($name:ident : $t:ty),* }; $($rest:tt)*) => (
    #[derive(PartialEq,Eq,Debug)]
    pub struct $state { $($name : $t,)* }

    state_impl!($machine, $($rest)*);
  );

  ($machine:ident,) =>  ( );
);

#[macro_export]
macro_rules! transitions_impl (
  ($machine:ident, $state:ident, $name:ident => $next:ident; $($rest:tt)* ) => (
    pub fn $name(&self) -> $machine<$next> {
       return $machine {
         state: $next
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
    SA {

    };

    SB {
      b: u8,
      c: bool
    };

    SC {

    } => {
      tr1 => SA;
    };
  });
  trace_macros!(false);

}
