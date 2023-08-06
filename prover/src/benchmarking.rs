#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Prover;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_benchmarking::impl_benchmark_test_suite;


benchmarks! {
    // Add individual benchmarks here
    generate_proof {
        /* code to set the initial state */
        // let accounts: Vec<T::AccountId> = generate_accounts::<T>(3);
        let caller: T::AccountId = whitelisted_caller();
        let origin = RawOrigin::Signed(caller);
            
        let chain_id_a = b"chain_id_a".to_vec();
        let pub_key_x_a = b"9923469788363394641447581010523031341756932740078311348094885040381333737036".to_vec();
        let pub_key_y_a = b"15136748386230632558663460691616320633534532960458260047240788628467741439965".to_vec();
        let s_key_a = b"27280609587350411503668598341887415318".to_vec();

        let tx_id = b"tx_id_1".to_vec();
        let msg = b"Hello world".to_vec();
        let hash = b"82f855acdbd9afcc6bf84b5ebd46a5c22863c292b351c3f2e1bcb15a36df19ca".to_vec();
        
        let chain_id_b = b"chain_id_b".to_vec();
        let pub_key_x_b = b"19623415798876512155667799412417316882995648600774116628180797896993935261123".to_vec();
        let pub_key_y_b = b"13430800622558173059766834386396396827007342311954539924157733552188600747062".to_vec();
        let s_key_b = b"252134077537012548843814694116065050410".to_vec();

        let chain_id_a = b"chain_id_a".to_vec();
        let pub_key_x_a = b"9923469788363394641447581010523031341756932740078311348094885040381333737036".to_vec();
        let pub_key_y_a = b"15136748386230632558663460691616320633534532960458260047240788628467741439965".to_vec();
        let s_key_a = b"27280609587350411503668598341887415318".to_vec();


    }: {
        /* code to test the function benchmarked */
        Prover::<T>::save_keys(origin.clone().into(), chain_id_b.clone(), pub_key_x_b, pub_key_y_b, s_key_b).expect("save_keys must not fail");
        Prover::<T>::gen_proof(origin.clone().into(), tx_id.clone(), chain_id_a.clone(), chain_id_b.clone(), msg.clone(), hash.clone()).expect("gen_proof must not fail");
    }
    verify {
        /* optional verification */
        if let Some(result) = Prover::<T>::get_proof(&tx_id){
            assert_eq!(result.tx_id, tx_id);
            assert_eq!(result.chain_id_a, chain_id_a);
            assert_eq!(result.chain_id_b, chain_id_b);
            assert_eq!(result.hash, hash.clone());
        }
    }
    
    impl_benchmark_test_suite!(Prover, crate::mock::new_test_ext(), crate::mock::Test,);
}

