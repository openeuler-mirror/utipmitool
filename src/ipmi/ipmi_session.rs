/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

impl IpmiContext {
        //TODO:ssn_params
    pub fn session_set_hostname(&mut self, hostname: String) {
        self.ssn_params.hostname = hostname;
    }

    pub fn session_set_username(&mut self, username: String) {
        self.ssn_params.username.fill(0);
        let len = username.len().min(16);
        self.ssn_params.username[..len].copy_from_slice(&username.as_bytes()[..len]);

    }

    pub fn session_set_password(&mut self, password: Option<&str>) {
        self.ssn_params.authcode_set.fill(0);
        if let Some(pass) = password {
            self.ssn_params.password = 1;
            let len = pass.len().min(IPMI_AUTHCODE_BUFFER_SIZE);
            self.ssn_params.authcode_set[..len].copy_from_slice(&pass.as_bytes()[..len]);
        } else {
            self.ssn_params.password = 0;
        }
    }

    pub fn session_set_privlvl(&mut self, level: u8) {
        self.ssn_params.privlvl = level;
    }

    pub fn session_set_lookupbit(&mut self, lookupbit: u8) {
        self.ssn_params.lookupbit = lookupbit;
    }

    pub fn session_set_cipher_suite_id(&mut self, cipher_suite_id: CipherSuiteIds) {
        self.ssn_params.cipher_suite_id = cipher_suite_id;
    }

    pub fn session_set_sol_escape_char(&mut self, sol_escape_char: char) {
        self.ssn_params.sol_escape_char = sol_escape_char;
    }

    pub fn session_set_kgkey(&mut self, kgkey: &[u8]) {
        self.ssn_params.kg[..IPMI_KG_BUFFER_SIZE].copy_from_slice(kgkey);
    }

    pub fn session_set_port(&mut self, port: i32) {
        self.ssn_params.port = port;
    }

    pub fn session_set_authtype(&mut self, authtype: u8) {
        if authtype == IPMI_SESSION_AUTHTYPE_NONE {
            self.ssn_params.authcode_set.fill(0);
            self.ssn_params.password = 0;
        }
        self.ssn_params.authtype_set = authtype;
    }

    pub fn session_set_timeout(&mut self, timeout: u32) {
        self.ssn_params.timeout = timeout;
    }

    pub fn session_set_retry(&mut self, retry: i32) {
        self.ssn_params.retry = retry;
    }

    //TODO:session

    pub fn session_cleanup(&mut self) {
        self.session = None;
    }

    pub fn cleanup(&mut self) {
        // TODO: ipmi_sdr_list_empty();
        if self.session.is_none() {
            return;
        }
        self.session = None;
    }
}