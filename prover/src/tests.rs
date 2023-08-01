use crate::mock::*;
use frame_support::assert_ok;
use env_logger;

fn get_signer(pubkey: &[u8; 32]) -> AccountId {
	test_utils::get_signer(pubkey)
}
fn setup_logger() {
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
    fn test_save_keys() {
        new_test_ext().execute_with(|| {
            let pubkey: [u8; 32] = [
                65, 89, 193, 118, 86, 172, 17, 149, 206, 160, 174, 75, 219, 151, 51, 235, 110, 135, 20,
                55, 147, 162, 106, 110, 143, 207, 57, 64, 67, 63, 203, 95,
            ];
            let signer = get_signer(&pubkey);
            let origin = RuntimeOrigin::signed(signer.clone());
            let chain_id_a = b"chain_id_a".to_vec();
            let pub_key_x = b"9923469788363394641447581010523031341756932740078311348094885040381333737036".to_vec();
            let pub_key_y = b"15136748386230632558663460691616320633534532960458260047240788628467741439965".to_vec();
            let s_key = b"27280609587350411503668598341887415318".to_vec();

            assert_ok!(Prover::save_keys(origin, chain_id_a, pub_key_x, pub_key_y, s_key));
        });
    }

    #[test]
    fn test_gen_proof() {
        new_test_ext().execute_with(|| {
            setup_logger();
            let pubkey: [u8; 32] = [
                65, 89, 193, 118, 86, 172, 17, 149, 206, 160, 174, 75, 219, 151, 51, 235, 110, 135, 20,
                55, 147, 162, 106, 110, 143, 207, 57, 64, 67, 63, 203, 95,
            ];
            let signer = get_signer(&pubkey);
            let origin = RuntimeOrigin::signed(signer.clone());
            
            let chain_id_a = b"chain_id_a".to_vec();
            let pub_key_x_a = b"9923469788363394641447581010523031341756932740078311348094885040381333737036".to_vec();
            let pub_key_y_a = b"15136748386230632558663460691616320633534532960458260047240788628467741439965".to_vec();
            let s_key_a = b"27280609587350411503668598341887415318".to_vec();

            assert_ok!(Prover::save_keys(origin.clone(), chain_id_a.clone(), pub_key_x_a, pub_key_y_a, s_key_a));            

            let chain_id_b = b"chain_id_b".to_vec();
            let pub_key_x_b = b"19623415798876512155667799412417316882995648600774116628180797896993935261123".to_vec();
            let pub_key_y_b = b"13430800622558173059766834386396396827007342311954539924157733552188600747062".to_vec();
            let s_key_b = b"252134077537012548843814694116065050410".to_vec();

            assert_ok!(Prover::save_keys(origin.clone(), chain_id_b.clone(), pub_key_x_b, pub_key_y_b, s_key_b));            

            let tx_id = b"tx_id_1".to_vec();
            let msg = b"Hello world".to_vec();
            let hash = b"82f855acdbd9afcc6bf84b5ebd46a5c22863c292b351c3f2e1bcb15a36df19ca".to_vec();

            assert_ok!(Prover::gen_proof(origin, tx_id, chain_id_a.clone(), chain_id_b.clone(), msg, hash));
        });
    }

// let arguments = [
// 	"9923469788363394641447581010523031341756932740078311348094885040381333737036", // PKX A
// 	"15136748386230632558663460691616320633534532960458260047240788628467741439965", // PKY A
// 	"27280609587350411503668598341887415318", // SK A
// 	"19623415798876512155667799412417316882995648600774116628180797896993935261123", // PKX B
// 	"13430800622558173059766834386396396827007342311954539924157733552188600747062", // PKY B
// 	"252134077537012548843814694116065050410", // SK B
// 	"174089066773056347813603871564866430402", // H :16
// 	"53687103630663083449284184610569132490", // H 16:
// 	"96231036770496792094352034275874832384", // MSG 16:
// 	"0", MSG 16:32
// 	"0", MSG 32:48
// 	"0" MSG 48:64
// ];