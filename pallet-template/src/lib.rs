#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super)fn deposit_event)]
    pub enum Event<T: Config> {
        //
    }

    #[pallet::error]
    pub enum Error<T> {
        //
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn do_something(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        //
    }
}
