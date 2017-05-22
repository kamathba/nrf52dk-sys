#![no_std]
#![no_main]

#![feature(asm)]

// TODO: remove?
#![allow(dead_code, unused_imports, unused_assignments, unused_variables)]

#[macro_use]
extern crate smooth_blue;

#[no_mangle]
pub unsafe extern "C" fn main() {
    // println!("Hello, world!");
    // Basic test that I can call the C code (and it is linked correctly)
    // NOTE: this probably still isn't a good test until I can verify linking
    let mut x = 0;
    x = smooth_blue::nrf_log_init(None);
    assert_eq!(x, 0);

    x = smooth_blue::app_timer_init();
    assert_eq!(x, 0);

    init_bt();

    // Mimic example
    loop {
        if !smooth_blue::nrf_log_frontend_dequeue() {
            smooth_blue::nrf_log_frontend_std_0(smooth_blue::NRF_LOG_LEVEL_ERROR as u8,
                                                "hello world\r\n\0".as_ptr());

            // This may not work, may need to init the softdevice first
            x = smooth_blue::sd_app_evt_wait();

            x += 1;

            x as usize;
        }
    }

    // x = bsp_init(smooth_blue::BSP_INIT_LED | smooth_blue::BSP_INIT_BUTTONS, 0);

    // x = bsp_btn_ble_init(0, 0);

    // g_erase_bonds = (startup_event == BSP_EVENT_CLEAR_BONDING_DATA);

}

pub unsafe fn init_sys() {}

pub unsafe fn init_bt() {
    // /**@brief Function for initializing the BLE stack.
    //  *
    //  * @details Initializes the SoftDevice and the BLE event interrupt.
    //  */
    // static void ble_stack_init(void)
    // {
    //     ret_code_t err_code;

    //     nrf_clock_lf_cfg_t clock_lf_cfg = NRF_CLOCK_LFCLKSRC;
    // Low frequency clock source to be used by the SoftDevice
    // #define NRF_CLOCK_LFCLKSRC      {.source        = NRF_CLOCK_LF_SRC_XTAL,            \
    //                                  .rc_ctiv       = 0,                                \
    //                                  .rc_temp_ctiv  = 0,                                \
    //                                  .xtal_accuracy = NRF_CLOCK_LF_XTAL_ACCURACY_20_PPM}

    let mut clk_cfg = smooth_blue::nrf_clock_lf_cfg_t {
        source: smooth_blue::NRF_CLOCK_LF_SRC_XTAL as u8,
        rc_ctiv: 0,
        rc_temp_ctiv: 0,
        xtal_accuracy: smooth_blue::NRF_CLOCK_LF_XTAL_ACCURACY_20_PPM as u8,
    };

    // //     // Initialize the SoftDevice handler module.
    // //     SOFTDEVICE_HANDLER_INIT(&clock_lf_cfg, NULL);
    //         // static uint32_t BLE_EVT_BUFFER[CEIL_DIV(BLE_STACK_EVT_MSG_BUF_SIZE, sizeof(uint32_t))];    \
    //         // uint32_t ERR_CODE;                                                                         \
    //         // ERR_CODE = softdevice_handler_init((CLOCK_SOURCE),                                         \
    //         //                                    BLE_EVT_BUFFER,                                         \
    //         //                                    sizeof(BLE_EVT_BUFFER),                                 \
    //         //                                    EVT_HANDLER);                                           \
    //         // APP_ERROR_CHECK(ERR_CODE);
    let mut event_buffer: [u8; 88] = [0; 88]; // 64 + 23 = 87, rounded up to next word
    let y = smooth_blue::softdevice_handler_init(&mut clk_cfg as
                                                 *mut smooth_blue::nrf_clock_lf_cfg_t,
                                                 &mut event_buffer[0] as *mut _ as
                                                 *mut smooth_blue::ctypes::c_void,
                                                 88,
                                                 None);
    assert_eq!(0, y);

    //     // Fetch the start address of the application RAM.
    //     uint32_t ram_start = 0;
    //     err_code = softdevice_app_ram_start_get(&ram_start);
    //     APP_ERROR_CHECK(err_code);

    let mut ram_start = 0u32;
    let y = smooth_blue::softdevice_app_ram_start_get(&mut ram_start);
    assert_eq!(0, y);

    //     // Overwrite some of the default configurations for the BLE stack.
    //     ble_cfg_t ble_cfg;
    //     memset(&ble_cfg, 0, sizeof(ble_cfg));
    //     ble_cfg.common_cfg.vs_uuid_cfg.vs_uuid_count = 0;
    //     err_code = sd_ble_cfg_set(BLE_COMMON_CFG_VS_UUID, &ble_cfg, ram_start);
    //     APP_ERROR_CHECK(err_code);

    let mut ble_cfg = core::mem::zeroed::<smooth_blue::ble_cfg_t>();
    let y = smooth_blue::sd_ble_cfg_set(smooth_blue::BLE_COMMON_CFGS::BLE_COMMON_CFG_VS_UUID as u32,
                                        &mut ble_cfg,
                                        ram_start);
    assert_eq!(0, y);

    //     // Configure the maximum number of connections.
    //     memset(&ble_cfg, 0, sizeof(ble_cfg));
    //     ble_cfg.gap_cfg.role_count_cfg.periph_role_count  = BLE_GAP_ROLE_COUNT_PERIPH_DEFAULT;
    //     ble_cfg.gap_cfg.role_count_cfg.central_role_count = 0;
    //     ble_cfg.gap_cfg.role_count_cfg.central_sec_count  = 0;
    let mut ble_cfg = core::mem::zeroed::<smooth_blue::ble_cfg_t>();
    *ble_cfg.gap_cfg.as_mut().role_count_cfg.as_mut() =
        smooth_blue::ble_gap_cfg_role_count_t {
            periph_role_count: smooth_blue::BLE_GAP_ROLE_COUNT_PERIPH_DEFAULT as u8,
            central_role_count: 0,
            central_sec_count: 0,
        };

    //     err_code = sd_ble_cfg_set(BLE_GAP_CFG_ROLE_COUNT, &ble_cfg, ram_start);
    //     APP_ERROR_CHECK(err_code);
    let y = smooth_blue::sd_ble_cfg_set(smooth_blue::BLE_GAP_CFGS::BLE_GAP_CFG_ROLE_COUNT as u32,
                                        &mut ble_cfg,
                                        ram_start);
    assert_eq!(y, 0);

    //     // Enable BLE stack.
    //     err_code = softdevice_enable(&ram_start);
    //     APP_ERROR_CHECK(err_code);
    let y = smooth_blue::softdevice_enable(&mut ram_start);
    assert_eq!(0, y);

    //     // Register with the SoftDevice handler module for BLE events.
    //     err_code = softdevice_ble_evt_handler_set(bt_evt);
    //     APP_ERROR_CHECK(err_code);
    let y = smooth_blue::softdevice_ble_evt_handler_set(Some(bt_evt));
    assert_eq!(0, y);

    //     // Register with the SoftDevice handler module for BLE events.
    //     err_code = softdevice_sys_evt_handler_set(sys_evt);
    //     APP_ERROR_CHECK(err_code);
    let y = smooth_blue::softdevice_sys_evt_handler_set(Some(sys_evt));
    assert_eq!(0, y);

    // gap_params_init()
    //////////////////////////////////////////////////////////////////////////
    // AJM TODO: This function is huge because there are some buffers that probably need to have a full lifetime.
    //     TODO refactor

    //    ret_code_t              err_code;
    //    ble_gap_conn_params_t   gap_conn_params;
    //    ble_gap_conn_sec_mode_t sec_mode;
    //
    //    BLE_GAP_CONN_SEC_MODE_SET_OPEN(&sec_mode);
    //    #define BLE_GAP_CONN_SEC_MODE_SET_OPEN(ptr)               do {(ptr)->sm = 1; (ptr)->lv = 1;} while(0)
    let mut sec_mode = core::mem::zeroed::<smooth_blue::ble_gap_conn_sec_mode_t>();
    sec_mode.set_sm(1);
    sec_mode.set_lv(1);

    //    err_code = sd_ble_gap_device_name_set(&sec_mode,
    //                                          (const uint8_t *)DEVICE_NAME,
    //                                          strlen(DEVICE_NAME));
    //    APP_ERROR_CHECK(err_code);

    let name = "RUST-BLE\0";
    let y = smooth_blue::sd_ble_gap_device_name_set(&mut sec_mode,
                                                    name.as_bytes().as_ptr(),
                                                    name.len() as u16);
    assert_eq!(0, y);

    //    /* YOUR_JOB: Use an appearance value matching the application's use case.
    //       err_code = sd_ble_gap_appearance_set(BLE_APPEARANCE_);
    //       APP_ERROR_CHECK(err_code); */
    //
    //    memset(&gap_conn_params, 0, sizeof(gap_conn_params));

    //    #define MIN_CONN_INTERVAL               MSEC_TO_UNITS(100, UNIT_1_25_MS)        /**< Minimum acceptable connection interval (0.1 seconds). */
    //    #define MAX_CONN_INTERVAL               MSEC_TO_UNITS(200, UNIT_1_25_MS)        /**< Maximum acceptable connection interval (0.2 second). */
    //    #define SLAVE_LATENCY                   0                                       /**< Slave latency. */
    //    #define CONN_SUP_TIMEOUT                MSEC_TO_UNITS(4000, UNIT_10_MS)         /**< Connection supervisory timeout (4 seconds). */
    //    #define MSEC_TO_UNITS(TIME, RESOLUTION) (((TIME) * 1000) / (RESOLUTION))

    //    gap_conn_params.min_conn_interval = MIN_CONN_INTERVAL;
    //    gap_conn_params.max_conn_interval = MAX_CONN_INTERVAL;
    //    gap_conn_params.slave_latency     = SLAVE_LATENCY;
    //    gap_conn_params.conn_sup_timeout  = CONN_SUP_TIMEOUT;

    let mut gap_conn_params = smooth_blue::ble_gap_conn_params_t {
        // min_conn_interval:  (100 * 1000) / (smooth_blue::UNIT_1_25_MS as u32) as u16,
        // max_conn_interval:  (200 * 1000) / (smooth_blue::UNIT_1_25_MS as u32) as u16,
        // slave_latency:      0,
        // conn_sup_timeout:   (4000 * 1000) / (smooth_blue::UNIT_10_MS as u32) as u16,
        min_conn_interval: 80,
        max_conn_interval: 160,
        slave_latency: 0,
        conn_sup_timeout: 400,
    };

    //    err_code = sd_ble_gap_ppcp_set(&gap_conn_params);
    //    APP_ERROR_CHECK(err_code);

    let y = smooth_blue::sd_ble_gap_ppcp_set(&mut gap_conn_params);
    assert_eq!(0, y);

    // void gatt_init(void)
    //////////////////////////////////////////////////////////////////////////
    //    static nrf_ble_gatt_t m_gatt;
    let mut m_gatt = core::mem::zeroed::<smooth_blue::nrf_ble_gatt_t>();

    //    ret_code_t err_code = nrf_ble_gatt_init(&m_gatt, NULL);
    //    APP_ERROR_CHECK(err_code);
    let y = smooth_blue::nrf_ble_gatt_init(&mut m_gatt, None);
    assert_eq!(0, y);

    // Interlude with ludes
    //    static ble_uuid_t m_adv_uuids[] = {{BLE_UUID_DEVICE_INFORMATION_SERVICE, BLE_UUID_TYPE_BLE}}; /**< Universally unique service identifiers. */
    let mut m_adv_uuids: [smooth_blue::ble_uuid_t; 1] =
        [smooth_blue::ble_uuid_t {
             uuid: smooth_blue::BLE_UUID_DEVICE_INFORMATION_SERVICE as u16,
             type_: smooth_blue::BLE_UUID_TYPE_BLE as u8,
         }];


    // void advertising_init
    //////////////////////////////////////////////////////////////////////////
    //    ret_code_t             err_code;
    //    ble_advdata_t          advdata;
    //    ble_adv_modes_config_t options;
    //
    //    // Build advertising data struct to pass into @ref ble_advertising_init.
    //    memset(&advdata, 0, sizeof(advdata));
    //    advdata.name_type               = BLE_ADVDATA_FULL_NAME;
    //    advdata.include_appearance      = true;
    //    advdata.flags                   = BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE;
    //    advdata.uuids_complete.uuid_cnt = sizeof(m_adv_uuids) / sizeof(m_adv_uuids[0]);
    //    advdata.uuids_complete.p_uuids  = m_adv_uuids;

    let mut advdata = core::mem::zeroed::<smooth_blue::ble_advdata_t>();

    advdata.name_type = smooth_blue::ble_advdata_name_type_t::BLE_ADVDATA_FULL_NAME;
    advdata.include_appearance = true;
    advdata.flags = smooth_blue::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8;
    advdata.uuids_complete.uuid_cnt = m_adv_uuids.len() as u16;
    advdata.uuids_complete.p_uuids = &mut m_adv_uuids[0];

    //    #define APP_ADV_INTERVAL                64
    //    #define APP_ADV_TIMEOUT_IN_SECONDS      180
    //    memset(&options, 0, sizeof(options));
    //    options.ble_adv_fast_enabled  = true;
    //    options.ble_adv_fast_interval = APP_ADV_INTERVAL;
    //    options.ble_adv_fast_timeout  = APP_ADV_TIMEOUT_IN_SECONDS;
    let mut options = core::mem::zeroed::<smooth_blue::ble_adv_modes_config_t>();
    options.ble_adv_fast_enabled = true;
    options.ble_adv_fast_interval = 64;
    options.ble_adv_fast_timeout = 180;

    //    err_code = ble_advertising_init(&advdata, NULL, &options, on_adv_evt, NULL);
    //    APP_ERROR_CHECK(err_code);
    let y = smooth_blue::ble_advertising_init(&advdata,
                                              core::ptr::null(),
                                              &options,
                                              Some(on_adv_evt),
                                              None);
    assert_eq!(0, y);

    // conn_params_init()
    //////////////////////////////////////////////////////////////////////////
    //    ret_code_t             err_code;
    //    ble_conn_params_init_t cp_init;
    //
    //    memset(&cp_init, 0, sizeof(cp_init));
    //
    //    cp_init.p_conn_params                  = NULL;
    //    cp_init.first_conn_params_update_delay = FIRST_CONN_PARAMS_UPDATE_DELAY;
    //    cp_init.next_conn_params_update_delay  = NEXT_CONN_PARAMS_UPDATE_DELAY;
    //    cp_init.max_conn_params_update_count   = MAX_CONN_PARAMS_UPDATE_COUNT;
    //    cp_init.start_on_notify_cccd_handle    = BLE_GATT_HANDLE_INVALID;
    //    cp_init.disconnect_on_fail             = false;
    //    cp_init.evt_handler                    = on_conn_params_evt;
    //    cp_init.error_handler                  = conn_params_error_handler;
    // #define APP_TIMER_TICKS(MS)                                \
    //             ((uint32_t)ROUNDED_DIV(                        \
    //             (MS) * (uint64_t)APP_TIMER_CLOCK_FREQ,         \
    //             1000 * (APP_TIMER_CONFIG_RTC_FREQUENCY + 1)))
    // #define FIRST_CONN_PARAMS_UPDATE_DELAY  APP_TIMER_TICKS(5000)
    // #define NEXT_CONN_PARAMS_UPDATE_DELAY   APP_TIMER_TICKS(30000)
    // #define MAX_CONN_PARAMS_UPDATE_COUNT    3

    let cp_init = smooth_blue::ble_conn_params_init_t {
        p_conn_params                  : core::ptr::null_mut(),
        first_conn_params_update_delay : (5 * 32768),
        next_conn_params_update_delay  : (30 * 32768),
        max_conn_params_update_count   : 3,
        start_on_notify_cccd_handle    : smooth_blue::BLE_GATT_HANDLE_INVALID as u16,
        disconnect_on_fail             : false,
        evt_handler                    : Some(on_conn_params_evt),
        error_handler                  : Some(conn_params_error_handler),
    };

    //    err_code = ble_conn_params_init(&cp_init);
    //    APP_ERROR_CHECK(err_code);
    let y = smooth_blue::ble_conn_params_init(&cp_init);
    assert_eq!(y, 0);

    // peer_manager_init()
    //////////////////////////////////////////////////////////////////////////
    //    ble_gap_sec_params_t sec_param;
    //    ret_code_t           err_code;
    //
    //    err_code = pm_init();
    //    APP_ERROR_CHECK(err_code);
    let y = smooth_blue::pm_init();
    assert_eq!(y, 0);

    //    memset(&sec_param, 0, sizeof(ble_gap_sec_params_t));
    //
    //    // Security parameters to be used for all security procedures.
    //    sec_param.bond           = SEC_PARAM_BOND;
    //    sec_param.mitm           = SEC_PARAM_MITM;
    //    sec_param.lesc           = SEC_PARAM_LESC;
    //    sec_param.keypress       = SEC_PARAM_KEYPRESS;
    //    sec_param.io_caps        = SEC_PARAM_IO_CAPABILITIES;
    //    sec_param.oob            = SEC_PARAM_OOB;
    //    sec_param.min_key_size   = SEC_PARAM_MIN_KEY_SIZE;
    //    sec_param.max_key_size   = SEC_PARAM_MAX_KEY_SIZE;
    //    sec_param.kdist_own.enc  = 1;
    //    sec_param.kdist_own.id   = 1;
    //    sec_param.kdist_peer.enc = 1;
    //    sec_param.kdist_peer.id  = 1;

    let mut kdist_own = core::mem::zeroed::<smooth_blue::ble_gap_sec_kdist_t>();
    kdist_own.set_enc(1);
    kdist_own.set_id(1);

    let mut kdist_peer = core::mem::zeroed::<smooth_blue::ble_gap_sec_kdist_t>();
    kdist_peer.set_enc(1);
    kdist_peer.set_id(1);

    let mut sec_param = smooth_blue::ble_gap_sec_params_t {
       _bitfield_1    : 0,
       min_key_size   : 7,
       max_key_size   : 16,
       kdist_own: kdist_own,
       kdist_peer: kdist_peer,
    };

    sec_param.set_bond(1);
    sec_param.set_io_caps(smooth_blue::BLE_GAP_IO_CAPS_NONE as u8);

    //    err_code = pm_sec_params_set(&sec_param);
    //    APP_ERROR_CHECK(err_code);
    let y = smooth_blue::pm_sec_params_set(&mut sec_param);
    assert_eq!(y, 0);

    //    err_code = pm_register(pm_evt_handler);
    //    APP_ERROR_CHECK(err_code);

    let y = smooth_blue::pm_register(Some(pm_evt_handler));
    assert_eq!(y, 0);

    advertising_start(true);


    //////////////////////////////////////////////////////////////////////////
    // AJM DONE HERE
    //////////////////////////////////////////////////////////////////////////
    loop {
        let _ = smooth_blue::sd_app_evt_wait();
    }

    // }

}

unsafe extern "C" fn bt_evt(p_ble_evt: *mut smooth_blue::ble_evt_t) {
    bkpt!();
}

unsafe extern "C" fn sys_evt(evt_id: u32) {
    bkpt!();
}

// static void on_adv_evt(ble_adv_evt_t ble_adv_evt)
// {
//     uint32_t err_code;
//     switch (ble_adv_evt)
//     {
//         case BLE_ADV_EVT_FAST:
//             err_code = bsp_indication_set(BSP_INDICATE_ADVERTISING);
//             APP_ERROR_CHECK(err_code);
//             break;
//         case BLE_ADV_EVT_IDLE:
//             sleep_mode_enter();
//             break;
//         default:
//             break;
//     }
// }

unsafe extern "C" fn on_adv_evt(ble_adv_evt: smooth_blue::ble_adv_evt_t) {
    bkpt!();
    use smooth_blue::ble_adv_evt_t::*;
    use smooth_blue::bsp_indication_t::*;

    match ble_adv_evt {
        BLE_ADV_EVT_FAST => {
            let y = smooth_blue::bsp_indication_set(BSP_INDICATE_ADVERTISING);
            assert_eq!(0, y);
        }
        BLE_ADV_EVT_IDLE => {
            sleep_mode_enter();
        }
        _ => {}
    }
}

// static void on_conn_params_evt(ble_conn_params_evt_t * p_evt)
// {
//     ret_code_t err_code;

//     if (p_evt->evt_type == BLE_CONN_PARAMS_EVT_FAILED)
//     {
//         err_code = sd_ble_gap_disconnect(m_conn_handle, BLE_HCI_CONN_INTERVAL_UNACCEPTABLE);
//         APP_ERROR_CHECK(err_code);
//     }
// }

unsafe extern "C" fn on_conn_params_evt(p_evt: *mut smooth_blue::ble_conn_params_evt_t) {
    use smooth_blue::ble_conn_params_evt_type_t::*;
    bkpt!();

    match (*p_evt).evt_type {
        BLE_CONN_PARAMS_EVT_FAILED => {
            // ?
        },
        _ => {}
    }
}

// static void conn_params_error_handler(uint32_t nrf_error)
// {
//     APP_ERROR_HANDLER(nrf_error);
// }
unsafe extern "C" fn conn_params_error_handler(nrf_error: u32) {
    bkpt!();
}

// static void sleep_mode_enter(void)
// {
//     uint32_t err_code = bsp_indication_set(BSP_INDICATE_IDLE);
//     APP_ERROR_CHECK(err_code);

//     // Prepare wakeup buttons.
//     err_code = bsp_btn_ble_sleep_mode_prepare();
//     APP_ERROR_CHECK(err_code);

//     // Go to system-off mode (this function will not return; wakeup will cause a reset).
//     err_code = sd_power_system_off();
//     APP_ERROR_CHECK(err_code);
// }

unsafe fn sleep_mode_enter() {
    let y = smooth_blue::bsp_indication_set(smooth_blue::bsp_indication_t_BSP_INDICATE_IDLE);
    assert_eq!(0, y);

    let y = smooth_blue::bsp_btn_ble_sleep_mode_prepare();
    assert_eq!(0, y);

    let y = smooth_blue::sd_power_system_off();
    assert_eq!(0, y);
}

// advertising_start()
//////////////////////////////////////////////////////////////////////////
//    if (erase_bonds == true)
//    {
//        delete_bonds();
//        // Advertising is started by PM_EVT_PEERS_DELETED_SUCEEDED evetnt
//    }
//    else
//    {
//        ret_code_t err_code = ble_advertising_start(BLE_ADV_MODE_FAST);
//
//        APP_ERROR_CHECK(err_code);
//    }
unsafe fn advertising_start(erase_bonds: bool) {
    if erase_bonds {
        delete_bonds();
        // Advertising is started by PM_EVT_PEERS_DELETED_SUCEEDED evetnt
    }
    else
    {
        use smooth_blue::ble_adv_mode_t::*;
        let y = smooth_blue::ble_advertising_start(BLE_ADV_MODE_FAST);
        assert_eq!(y, 0);
    }
}

// static void delete_bonds(void)
// {
//     ret_code_t err_code;

//     NRF_LOG_INFO("Erase bonds!\r\n");

//     err_code = pm_peers_delete();
//     APP_ERROR_CHECK(err_code);
// }

unsafe fn delete_bonds() {
    let y = smooth_blue::pm_peers_delete();
    assert_eq!(y, 0);
}

// static void pm_evt_handler(pm_evt_t const * p_evt)
// {
//     ret_code_t err_code;
unsafe extern "C" fn pm_evt_handler(p_evt: *const smooth_blue::pm_evt_t) {
    use smooth_blue::pm_evt_id_t::*;

//     switch (p_evt->evt_id)
//     {
    match (*p_evt).evt_id {
//         case PM_EVT_BONDED_PEER_CONNECTED:
//         {
//             NRF_LOG_INFO("Connected to a previously bonded device.\r\n");
//         } break;
        PM_EVT_BONDED_PEER_CONNECTED => {}, // todo log

//         case PM_EVT_CONN_SEC_SUCCEEDED:
//         {
//             NRF_LOG_INFO("Connection secured: role: %d, conn_handle: 0x%x, procedure: %d.\r\n",
//                          ble_conn_state_role(p_evt->conn_handle),
//                          p_evt->conn_handle,
//                          p_evt->params.conn_sec_succeeded.procedure);
//         } break;
        PM_EVT_CONN_SEC_SUCCEEDED => {}, // todo log

//         case PM_EVT_CONN_SEC_FAILED:
//         {
//             /* Often, when securing fails, it shouldn't be restarted, for security reasons.
//              * Other times, it can be restarted directly.
//              * Sometimes it can be restarted, but only after changing some Security Parameters.
//              * Sometimes, it cannot be restarted until the link is disconnected and reconnected.
//              * Sometimes it is impossible, to secure the link, or the peer device does not support it.
//              * How to handle this error is highly application dependent. */
//         } break;
        PM_EVT_CONN_SEC_FAILED => {}, // todo, error handling

//         case PM_EVT_CONN_SEC_CONFIG_REQ:
//         {
//             // Reject pairing request from an already bonded peer.
//             pm_conn_sec_config_t conn_sec_config = {.allow_repairing = false};
//             pm_conn_sec_config_reply(p_evt->conn_handle, &conn_sec_config);
//         } break;
        PM_EVT_CONN_SEC_CONFIG_REQ => {}, // todo, handle global interaction

//         case PM_EVT_STORAGE_FULL:
//         {
//             // Run garbage collection on the flash.
//             err_code = fds_gc();
//             if (err_code == FDS_ERR_BUSY || err_code == FDS_ERR_NO_SPACE_IN_QUEUES)
//             {
//                 // Retry.
//             }
//             else
//             {
//                 APP_ERROR_CHECK(err_code);
//             }
//         } break;
        PM_EVT_STORAGE_FULL => {
            let y = smooth_blue::fds_gc();
            // todo retry
            assert_eq!(y, 0);
        },

//         case PM_EVT_PEERS_DELETE_SUCCEEDED:
//         {
//             advertising_start(false);
//         } break;
        PM_EVT_PEERS_DELETE_SUCCEEDED => {
            advertising_start(false);
        }

//         case PM_EVT_LOCAL_DB_CACHE_APPLY_FAILED:
//         {
//             // The local database has likely changed, send service changed indications.
//             pm_local_database_has_changed();
//         } break;
        PM_EVT_LOCAL_DB_CACHE_APPLY_FAILED => {
            smooth_blue::pm_local_database_has_changed();
        },

//         case PM_EVT_PEER_DATA_UPDATE_FAILED:
//         {
//             // Assert.
//             APP_ERROR_CHECK(p_evt->params.peer_data_update_failed.error);
//         } break;
        PM_EVT_PEER_DATA_UPDATE_FAILED => {
            // todo assert
        },

//         case PM_EVT_PEER_DELETE_FAILED:
//         {
//             // Assert.
//             APP_ERROR_CHECK(p_evt->params.peer_delete_failed.error);
//         } break;
        PM_EVT_PEER_DELETE_FAILED => {
            // todo assert
        },

//         case PM_EVT_PEERS_DELETE_FAILED:
//         {
//             // Assert.
//             APP_ERROR_CHECK(p_evt->params.peers_delete_failed_evt.error);
//         } break;
        PM_EVT_PEERS_DELETE_FAILED => {
            // todo assert
        },

//         case PM_EVT_ERROR_UNEXPECTED:
//         {
//             // Assert.
//             APP_ERROR_CHECK(p_evt->params.error_unexpected.error);
//         } break;
        PM_EVT_ERROR_UNEXPECTED => {
            // todo assert
        },

//         case PM_EVT_CONN_SEC_START:
//         case PM_EVT_PEER_DATA_UPDATE_SUCCEEDED:
//         case PM_EVT_PEER_DELETE_SUCCEEDED:
//         case PM_EVT_LOCAL_DB_CACHE_APPLIED:
//         case PM_EVT_SERVICE_CHANGED_IND_SENT:
//         case PM_EVT_SERVICE_CHANGED_IND_CONFIRMED:
//         default:
//             break;
//     }
        _ => {}
    }




// }
}