#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, IsType, Randomness},
	};
	use frame_system::pallet_prelude::{OriginFor, *};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type MaxKittiesOwned: Get<u32>;

		type KittyRandomness: Randomness<Self::Hash, u32>;
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Copy)]
	pub enum Gender {
		Male,
		Female,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Copy)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: T::AccountId,
	}

	#[pallet::storage]
	pub(super) type CountForKitties<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], Kitty<T>>;

	#[pallet::storage]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<[u8; 16], T::MaxKittiesOwned>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created { kitty: [u8; 16], owner: T::AccountId },
		Transferred { from: T::AccountId, to: T::AccountId, kitty: [u8; 16] },
		PriceSet { kitty: [u8; 16], price: Option<BalanceOf<T>> },
		Sold { seller: T::AccountId, buyer: T::AccountId, kitty: [u8; 16], price: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		TooManyOwned,
		DuplicateKitty,
		Overflow,
		NoKitty,
		NotOwner,
		TransferToSelf,
		BidPriceTooLow,
		NotForSale,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (kitty_gen_dna, gender) = Self::gen_dna();

			let _ = Self::mint(&sender, kitty_gen_dna, gender);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: [u8; 16],
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == from, Error::<T>::NotOwner);
			let _ = Self::do_transfer(kitty_id, to);
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: [u8; 16],
			price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == sender, Error::<T>::NotOwner);
			kitty.price = price;
			Kitties::<T>::insert(kitty_id, kitty);
			Self::deposit_event(Event::PriceSet { kitty: kitty_id, price });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn gen_dna() -> ([u8; 16], Gender) {
			let random = T::KittyRandomness::random(&b"dna"[..]).0;

			let unique_payload = (
				random,
				frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
				frame_system::Pallet::<T>::block_number(),
			);

			let encoded_payload = unique_payload.encode();
			let hash = frame_support::Hashable::blake2_128(&encoded_payload);

			if hash[0] % 2 == 0 {
				(hash, Gender::Male)
			} else {
				(hash, Gender::Female)
			}
		}

		pub fn mint(
			owner: &T::AccountId,
			dna: [u8; 16],
			gender: Gender,
		) -> Result<[u8; 16], DispatchError> {
			let kitty = Kitty::<T> { dna, price: None, gender, owner: owner.clone() };
			ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::DuplicateKitty);
			let count = CountForKitties::<T>::get();
			let new_count = count.checked_add(1).ok_or(Error::<T>::Overflow)?;
			KittiesOwned::<T>::try_append(&owner, kitty.dna)
				.map_err(|_| Error::<T>::TooManyOwned)?;

			Kitties::<T>::insert(kitty.dna, kitty);
			CountForKitties::<T>::put(new_count);

			Self::deposit_event(Event::Created { kitty: dna, owner: owner.clone() });
			Ok(dna)
		}

		pub fn do_transfer(kitty_id: [u8; 16], to: T::AccountId) -> DispatchResult {
			let mut kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			let from = kitty.owner;

			ensure!(from != to, Error::<T>::TransferToSelf);

			let mut from_owned = KittiesOwned::<T>::get(&from);

			if let Some(ind) = from_owned.iter().position(|&id| id == kitty_id) {
				from_owned.swap_remove(ind);
			} else {
				return Err(Error::<T>::NoKitty.into())
			}

			let mut to_owned = KittiesOwned::<T>::get(&to);

			let _ = to_owned.try_push(kitty_id).map_err(|_| Error::<T>::TooManyOwned);

			kitty.owner = to.clone();
			kitty.price = None;

			Kitties::<T>::insert(&kitty_id, kitty);
			KittiesOwned::<T>::insert(&to, to_owned);
			KittiesOwned::<T>::insert(&from, from_owned);

			Self::deposit_event(Event::Transferred { from, to, kitty: kitty_id });

			Ok(())
		}
		pub fn do_buy_kitty(
			kitty_id: [u8; 16],
			to: T::AccountId,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			let from = kitty.owner;
			ensure!(to != from, Error::<T>::TransferToSelf);

			if let Some(price) = kitty.price {
				ensure!(bid_price >= price, Error::<T>::BidPriceTooLow);
				T::Currency::transfer(
					&to,
					&from,
					price,
					frame_support::traits::ExistenceRequirement::KeepAlive,
				)?;
				let _ = Self::do_transfer(kitty_id, to);
			} else {
				return Err(Error::<T>::NotForSale.into())
			}

			Ok(())
		}
	}
}
