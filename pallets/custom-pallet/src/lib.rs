#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
use crate::weights::WeightInfo;

#[frame::pallet]
pub mod pallet {
    use super::*;
    use frame::{
        deps::frame_support::traits::{Currency, Hooks},
        deps::sp_runtime::DispatchResult,
        prelude::{ensure_root, *},
    };

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type CounterMaxValue: Get<u32>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;

        type Currency: Currency<Self::AccountId>;
    }

    /// Hook类型枚举，用于区分不同的hook函数
    #[derive(
        Clone,
        Copy,
        PartialEq,
        Eq,
        Debug,
        codec::Encode,
        codec::Decode,
        codec::DecodeWithMemTracking,
        codec::MaxEncodedLen,
        scale_info::TypeInfo,
    )]
    pub enum HookType {
        /// on_initialize hook
        OnInitialize,
        /// on_finalize hook
        OnFinalize,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CounterValueSet {
            counter_value: u32,
        },
        /// A user has successfully incremented the counter.
        CounterIncremented {
            /// The new value set.
            counter_value: u32,
            /// The account who incremented the counter.
            who: T::AccountId,
            /// The amount by which the counter was incremented.
            incremented_amount: u32,
        },
        /// A user has successfully decremented the counter.
        CounterDecremented {
            /// The new value set.
            counter_value: u32,
            /// The account who decremented the counter.
            who: T::AccountId,
            /// The amount by which the counter was decremented.
            decremented_amount: u32,
        },
        /// 转账成功事件
        TransferSucceeded {
            from: T::AccountId,
            to: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// 调用Hooks
        HooksFuncCalled {
            hook_type: HookType,
            block_number: BlockNumberFor<T>,
        },
    }

    #[pallet::storage]
    pub type CounterValue<T> = StorageValue<_, u32>;

    #[pallet::storage]
    pub type UserInteractions<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u32>;

    #[pallet::error]
    pub enum Error<T> {
        /// The counter value exceeds the maximum allowed value.
        CounterValueExceedsMax,
        /// The counter value cannot be decremented below zero.
        CounterValueBelowZero,
        /// Overflow occurred in the counter.
        CounterOverflow,
        /// Overflow occurred in user interactions.
        UserInteractionOverflow,

        /// transfer
        InsufficientBalance,
        ZeroAmount,
        TransferToSelf,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        // #[pallet::weight(0)]
        #[pallet::weight(T::WeightInfo::set_counter_value())]
        pub fn set_counter_value(origin: OriginFor<T>, new_value: u32) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(
                new_value <= T::CounterMaxValue::get(),
                Error::<T>::CounterValueExceedsMax
            );

            CounterValue::<T>::put(new_value);
            Self::deposit_event(Event::CounterValueSet {
                counter_value: new_value,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        // #[pallet::weight(0)]
        #[pallet::weight(T::WeightInfo::increment())]
        pub fn increment(origin: OriginFor<T>, amount_to_increment: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_value = CounterValue::<T>::get().unwrap_or(0);

            let new_value = current_value
                .checked_add(amount_to_increment)
                .ok_or(Error::<T>::CounterOverflow)?;

            ensure!(
                new_value <= T::CounterMaxValue::get(),
                Error::<T>::CounterValueExceedsMax
            );

            CounterValue::<T>::put(new_value);

            UserInteractions::<T>::try_mutate(&who, |interactions| -> Result<_, Error<T>> {
                let new_interactions = interactions
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or(Error::<T>::UserInteractionOverflow)?;
                *interactions = Some(new_interactions); // Store the new value.

                Ok(())
            })?;

            Self::deposit_event(Event::<T>::CounterIncremented {
                counter_value: new_value,
                who,
                incremented_amount: amount_to_increment,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        // #[pallet::weight(0)]
        #[pallet::weight(T::WeightInfo::decrement())]
        pub fn decrement(origin: OriginFor<T>, amount_to_decrement: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_value = CounterValue::<T>::get().unwrap_or(0);

            let new_value = current_value
                .checked_sub(amount_to_decrement)
                .ok_or(Error::<T>::CounterValueBelowZero)?;

            CounterValue::<T>::put(new_value);

            // 更新次数
            UserInteractions::<T>::try_mutate(&who, |interactions| -> Result<_, Error<T>> {
                let new_interactions = interactions
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or(Error::<T>::UserInteractionOverflow)?;
                *interactions = Some(new_interactions); // Store the new value.

                Ok(())
            })?;

            Self::deposit_event(Event::<T>::CounterDecremented {
                counter_value: new_value,
                who,
                decremented_amount: amount_to_decrement,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn reset_counter(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            <CounterValue<T>>::put(0u32);
            Self::deposit_event(Event::CounterValueSet { counter_value: 0 });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            // 输入验证
            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
            ensure!(from != dest, Error::<T>::TransferToSelf);

            // 检查余额是否足够
            let from_balance = T::Currency::free_balance(&from);
            ensure!(from_balance >= amount, Error::<T>::InsufficientBalance);

            // 执行转账
            T::Currency::transfer(&from, &dest, amount, ExistenceRequirement::KeepAlive)?;

            Self::deposit_event(Event::TransferSucceeded {
                from,
                to: dest,
                amount,
            });

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 在每个区块开始时调用的 hook
        fn on_initialize(_block_number: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
            // 获取当前计数器值
            let current_counter = CounterValue::<T>::get().unwrap_or(0);

            // 如果计数器值为 0，则初始化为 1
            if current_counter == 0 {
                CounterValue::<T>::put(1u32);

                // 记录初始化事件
                Self::deposit_event(Event::CounterValueSet { counter_value: 1 });
                Self::deposit_event(Event::HooksFuncCalled {
                    hook_type: HookType::OnInitialize,
                    block_number: _block_number,
                });

                // 返回执行的权重：一次存储写入
                T::DbWeight::get().writes(1)
            } else {
                // 没有执行任何操作，返回最小权重
                Weight::zero()
            }
        }

        /// 在每个区块结束时调用的 hook
        fn on_finalize(_block_number: BlockNumberFor<T>) {
            // 获取当前计数器值
            let current_counter = CounterValue::<T>::get().unwrap_or(0);

            // 如果计数器值超过最大值的一半，发出警告事件
            let max_value = T::CounterMaxValue::get();
            if current_counter > max_value / 2 {
                // 通过事件系统通知而非直接日志
                Self::deposit_event(Event::CounterValueSet {
                    counter_value: current_counter,
                });
                Self::deposit_event(Event::HooksFuncCalled {
                    hook_type: HookType::OnFinalize,
                    block_number: _block_number,
                });
            }

            // 检查是否有用户交互次数异常高的账户
            // 这里可以存储需要监控的账户，而不是直接日志
            // 实际应用中可能需要更复杂的逻辑
        }
    }
}
