use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::storage_types::{DataKey, VestingSchedule,StakeInfo,PoolInfo};
use soroban_sdk::token::{self, Interface as _};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;
use soroban_sdk::{contract, contractimpl, Address, Env, String, Map};


///staking kodları

// Kontrat veri yapısı
#[contract]
pub struct StakingRewards;

// Verileri saklamak için kullanılacak anahtarlar
const POOL_INFO_KEY: &str = "pool_info";
const STAKES_KEY: &str = "stakes";
const ADMIN_KEY: &str = "admin";

// Özel olayları yayınlamak için yardımcı fonksiyon
fn emit_event(e: &Env, event_type: &str, user: &Address, amount: i128) {
    e.events().publish((event_type, user.clone(), amount), ());
}
//////


fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

// Bir hesabın dondurulup dondurulmadığını kontrol eden yardımcı fonksiyon
fn is_account_frozen(e: &Env, account: &Address) -> bool {
    let key = DataKey::Frozen(account.clone());
    e.storage().instance().get::<_, bool>(&key).unwrap_or(false)
}

// Özel olayları yayınlamak için yardımcı fonksiyon
fn emit_custom_event(e: &Env, event_type: &str, admin: Address, account: Address) {
    e.events().publish((event_type, admin, account), ());
}

// Vesting planı işlemleri için yardımcı fonksiyonlar
fn read_vesting_schedule(e: &Env, beneficiary: &Address) -> Option<VestingSchedule> {
    let key = DataKey::VestingSchedule(beneficiary.clone());
    e.storage().instance().get(&key)
}

fn write_vesting_schedule(e: &Env, beneficiary: &Address, schedule: &VestingSchedule) {
    let key = DataKey::VestingSchedule(beneficiary.clone());
    e.storage().instance().set(&key, schedule);
}

fn remove_vesting_schedule(e: &Env, beneficiary: &Address) {
    let key = DataKey::VestingSchedule(beneficiary.clone());
    e.storage().instance().remove(&key);
}

fn get_claimable_amount(e: &Env, beneficiary: &Address) -> i128 {
    if let Some(schedule) = read_vesting_schedule(e, beneficiary) {
        let current_ledger = e.ledger().sequence();
        
        // Eğer henüz başlangıç zamanına gelmemişse veya cliff zamanından önceyse
        if current_ledger < schedule.start_ledger || 
           (schedule.cliff_ledger > 0 && current_ledger < schedule.cliff_ledger) {
            return 0;
        }
        
        // Eğer bitiş zamanını geçtiyse, kalan tüm miktar çekilebilir
        if current_ledger >= schedule.end_ledger {
            return schedule.total_amount - schedule.claimed_amount;
        }
        
        // Lineer vesting: Geçen zamana orantılı olarak token miktarı hesaplanır
        let total_vesting_time = schedule.end_ledger - schedule.start_ledger;
        let elapsed_time = current_ledger - schedule.start_ledger;
        
        let claimable_amount = schedule.total_amount * elapsed_time as i128 / total_vesting_time as i128;
        
        // Şimdiye kadar çekilen miktarı çıkaralım
        if claimable_amount <= schedule.claimed_amount {
            return 0;
        }
        
        return claimable_amount - schedule.claimed_amount;
    }
    
    0 // Vesting planı yoksa çekilebilir miktar 0
}

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(e: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);
        if decimal > u8::MAX.into() {
            panic!("Decimal must fit in a u8");
        }

        write_metadata(
            &e,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        )
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().mint(admin, to, amount);
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_administrator(&e, &new_admin);
        TokenUtils::new(&e).events().set_admin(admin, new_admin);
    }

    // Bir hesabı dondur (sadece yönetici yapabilir)
    pub fn freeze_account(e: Env, account: Address) {
        // Sadece yönetici hesapları dondurabilir
        let admin = read_administrator(&e);
        admin.require_auth();

        // Kontrat örneğinin TTL süresini uzat
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Hesabı dondurulmuş olarak ayarla
        let key = DataKey::Frozen(account.clone());
        e.storage().instance().set(&key, &true);

       // Dondurma olayını yayınla
       emit_custom_event(&e, "freeze_account", admin, account);
    }

    // Bir hesabın dondurulmasını kaldır (sadece yönetici yapabilir)
    pub fn unfreeze_account(e: Env, account: Address) {
        // Sadece yönetici hesapların dondurulmasını kaldırabilir
        let admin = read_administrator(&e);
        admin.require_auth();

        // Kontrat örneğinin TTL süresini uzat
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Dondurulmuş durumu kaldır
        let key = DataKey::Frozen(account.clone());
        e.storage().instance().remove(&key);

        // Dondurma kaldırma olayını yayınla
        emit_custom_event(&e, "unfreeze_account", admin, account);
    }

    pub fn create_vesting(
        e: Env,
        beneficiary: Address,
        amount: i128,
        start_ledger: u32,
        cliff_ledger: u32,
        end_ledger: u32,
    ) {
        let admin = read_administrator(&e);
        admin.require_auth();
        
        check_nonnegative_amount(amount);
        
        // Parametrelerin mantıklı olduğunu kontrol et
        if end_ledger <= start_ledger {
            panic!("Bitiş ledger'ı başlangıç ledger'ından büyük olmalıdır");
        }
        
        if cliff_ledger > 0 && cliff_ledger < start_ledger {
            panic!("Cliff ledger'ı başlangıç ledger'ından küçük olamaz");
        }
        
        // Yöneticinin yeterli token'a sahip olduğunu kontrol et
        let admin_balance = read_balance(&e, admin.clone());
        if admin_balance < amount {
            panic!("Yönetici vesting için yeterli token'a sahip değil");
        }
        
        // TTL süresini uzat
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        // Vesting planını kaydet
        let schedule = VestingSchedule {
            beneficiary: beneficiary.clone(),
            total_amount: amount,
            claimed_amount: 0,
            start_ledger,
            cliff_ledger,
            end_ledger,
        };
        
        write_vesting_schedule(&e, &beneficiary, &schedule);
        
        // Vesting için ayrılan token'ları yöneticiden al (dondurulmuş olarak saklanacak)
        spend_balance(&e, admin.clone(), amount);
        receive_balance(&e, beneficiary.clone(), amount);
        
        // Hesabı otomatik olarak dondur
        let key = DataKey::Frozen(beneficiary.clone());
        e.storage().instance().set(&key, &true);
        
        // Vesting oluşturma olayını yayınla
        emit_custom_event(&e, "create_vesting", admin, beneficiary);
    }
    
    // Vesting planından token'ları talep et (sadece faydalanıcı yapabilir)
    pub fn claim_vesting(e: Env, beneficiary: Address) -> i128 {
        beneficiary.require_auth();
        
        // TTL süresini uzat
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        // Çekilebilir miktarı hesapla
        let claimable_amount = get_claimable_amount(&e, &beneficiary);
        
        if claimable_amount <= 0 {
            panic!("Çekilebilir token bulunmamaktadır");
        }
        
        // Vesting planını güncelle
        if let Some(mut schedule) = read_vesting_schedule(&e, &beneficiary) {
            schedule.claimed_amount += claimable_amount;
            
            // Eğer tüm tokenlar çekildiyse vesting planını kaldır ve hesabı çöz
            if schedule.claimed_amount >= schedule.total_amount {
                remove_vesting_schedule(&e, &beneficiary);
                
                // Hesabın dondurulmasını kaldır
                let key = DataKey::Frozen(beneficiary.clone());
                e.storage().instance().remove(&key);
            } else {
                write_vesting_schedule(&e, &beneficiary, &schedule);
            }
            
            // Donmuş token'ların çözülmesini sağla
            let admin = read_administrator(&e);
            
            // Özel bir olay yayınla
            emit_custom_event(&e, "claim_vesting", admin, beneficiary);
            
            return claimable_amount;
        }
        
        panic!("Vesting planı bulunamadı");
    }
    
    // Faydalanıcı için vesting planı bilgilerini getir
    pub fn get_vesting_info(e: Env, beneficiary: Address) -> Option<VestingSchedule> {
        // TTL süresini uzat
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        read_vesting_schedule(&e, &beneficiary)
    }
    
    // Çekilebilir vesting miktarını hesapla
    pub fn get_claimable_vesting(e: Env, beneficiary: Address) -> i128 {
        // TTL süresini uzat
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        get_claimable_amount(&e, &beneficiary)
    }
    
    // Bir vesting planını iptal et (sadece admin yapabilir)
    pub fn revoke_vesting(e: Env, beneficiary: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();
        
        // TTL süresini uzat
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        if let Some(schedule) = read_vesting_schedule(&e, &beneficiary) {
            // Henüz çekilmemiş token'ları admin'e geri transfer et
            let unclaimed_amount = schedule.total_amount - schedule.claimed_amount;
            if unclaimed_amount > 0 {
                spend_balance(&e, beneficiary.clone(), unclaimed_amount);
                receive_balance(&e, admin.clone(), unclaimed_amount);
            }
            
            // Vesting planını kaldır
            remove_vesting_schedule(&e, &beneficiary);
            
            // Hesabın dondurulmasını kaldır
            let key = DataKey::Frozen(beneficiary.clone());
            e.storage().instance().remove(&key);
            
            // İptal etme olayını yayınla
            emit_custom_event(&e, "revoke_vesting", admin, beneficiary);
        } else {
            panic!("Vesting planı bulunamadı");
        }
    }

     
///Staking kodları
    pub fn initialize_staking(
        e: Env,
        admin: Address,
        token_id: Address,
        reward_token_id: Address,
        reward_rate: u32,
        min_stake_duration: u32,
    ) {
        // Kontratın sadece bir kez başlatılabilmesini sağla
        if e.storage().instance().has(&ADMIN_KEY) {
            panic!("Contract already initialized");
        }
        
        // Admin adresini kaydet
        e.storage().instance().set(&ADMIN_KEY, &admin);
        
        // Havuz bilgilerini kaydet
        let pool_info = PoolInfo {
            token_id,
            reward_token_id,
            reward_rate,
            total_staked: 0,
            min_stake_duration,
        };
        e.storage().instance().set(&POOL_INFO_KEY, &pool_info);
        
        // Boş bir stake haritası oluştur
        let stakes: Map<Address, StakeInfo> = Map::new(&e);
        e.storage().instance().set(&STAKES_KEY, &stakes);
        
        // TTL süresini uzat
        e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        // Başlatma olayını yayınla
        emit_event(&e, "initialize", &admin, 0);
    }
    
    // Ödül oranını güncelleme (sadece admin yapabilir)
    pub fn update_reward_rate(e: Env, new_rate: u32) {
        // Admin kontrolü
        let admin: Address = e.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Havuz bilgilerini al ve güncelle
        let mut pool_info: PoolInfo = e.storage().instance().get(&POOL_INFO_KEY).unwrap();
        pool_info.reward_rate = new_rate;
        e.storage().instance().set(&POOL_INFO_KEY, &pool_info);
        
        // TTL süresini uzat
        e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        // Oranı güncelleme olayını yayınla
        emit_event(&e, "update_rate", &admin, new_rate as i128);
    }
    
    // Minimum stake süresini güncelleme (sadece admin yapabilir)
    pub fn update_min_stake_duration(e: Env, new_duration: u32) {
        // Admin kontrolü
        let admin: Address = e.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Havuz bilgilerini al ve güncelle
        let mut pool_info: PoolInfo = e.storage().instance().get(&POOL_INFO_KEY).unwrap();
        pool_info.min_stake_duration = new_duration;
        e.storage().instance().set(&POOL_INFO_KEY, &pool_info);
        
        // TTL süresini uzat
        e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        // Minimum süreyi güncelleme olayını yayınla
        emit_event(&e, "update_min_duration", &admin, new_duration as i128);
    }
    
    // Tokenları stake etme fonksiyonu
    pub fn stake(e: Env, user: Address, amount: i128) {
        // Kullanıcının yetkilendirmesini kontrol et
        user.require_auth();
        
        // Negatif miktar kontrolü
        if amount <= 0 {
            panic!("Stake amount must be positive");
        }
        
        // Havuz bilgilerini al
        let mut pool_info: PoolInfo = e.storage().instance().get(&POOL_INFO_KEY).unwrap();
        
        // Kullanıcının bakiyesini kontrol et
        let user_balance = read_balance(&e, user.clone());
        if user_balance < amount {
            panic!("Insufficient balance");
        }
        
        // Token transferini doğrudan depolama işlemleriyle yap (re-entry önlemek için)
        spend_balance(&e, user.clone(), amount);
        receive_balance(&e, e.current_contract_address(), amount);
        
        // Mevcut stake bilgilerini al veya yeni oluştur
        let mut stakes: Map<Address, StakeInfo> = e.storage().instance().get(&STAKES_KEY).unwrap();
        
        let current_ledger = e.ledger().sequence();
        
        if let Some(mut stake_info) = stakes.get(user.clone()) {
            // Eğer kullanıcının mevcut stake'i varsa, önce bekleyen ödülleri hesapla ve stake'i güncelle
            let pending_reward = Self::calculate_reward(&e, &user, &stake_info, &pool_info);
            
            // Varsa ödülleri gönder - burada da doğrudan depolama kullan
            if pending_reward > 0 {
                // Token transferi yerine direkt bakiye güncelle
                spend_balance(&e, e.current_contract_address(), pending_reward);
                receive_balance(&e, user.clone(), pending_reward);
                
                // Ödül çekme olayını yayınla
                emit_event(&e, "claim_reward", &user, pending_reward);
            }
            
            // Stake bilgisini güncelle
            stake_info.amount += amount;
            stake_info.last_claim_ledger = current_ledger;
            stakes.set(user.clone(), stake_info);
        } else {
            // Yeni stake oluştur
            let stake_info = StakeInfo {
                amount,
                since_ledger: current_ledger,
                last_claim_ledger: current_ledger,
            };
            stakes.set(user.clone(), stake_info);
        }
        
        // Toplam stake miktarını güncelle
        pool_info.total_staked += amount;
        
        // Güncellenmiş bilgileri kaydet
        e.storage().instance().set(&POOL_INFO_KEY, &pool_info);
        e.storage().instance().set(&STAKES_KEY, &stakes);
        
        // TTL süresini uzat
        e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        // Stake olayını yayınla
        emit_event(&e, "stake", &user, amount);
    }
    
    // Ödül hesaplama (internal fonksiyon)
    fn calculate_reward(e: &Env, user: &Address, stake_info: &StakeInfo, pool_info: &PoolInfo) -> i128 {
        let current_ledger = e.ledger().sequence();
        
        // Son çekimden bu yana geçen ledger sayısı
        let ledgers_passed = current_ledger - stake_info.last_claim_ledger;
        
        // Ödülü hesapla: stake miktarı * ödül oranı * geçen ledger sayısı / 10000
        // (10000 bölmesi ödül oranını daha hassas ayarlamaya olanak tanır)
        (stake_info.amount * pool_info.reward_rate as i128 * ledgers_passed as i128) / 10000
    }
    
    // Ödül çekme fonksiyonu
    pub fn claim_rewards(e: Env, user: Address) -> i128 {
        // Kullanıcının yetkilendirmesini kontrol et
        user.require_auth();
        
        // Havuz ve stake bilgilerini al
        let pool_info: PoolInfo = e.storage().instance().get(&POOL_INFO_KEY).unwrap();
        let mut stakes: Map<Address, StakeInfo> = e.storage().instance().get(&STAKES_KEY).unwrap();
        
        // Kullanıcının stake bilgisini kontrol et
        if let Some(mut stake_info) = stakes.get(user.clone()) {
            // Bekleyen ödülü hesapla
            let reward = Self::calculate_reward(&e, &user, &stake_info, &pool_info);
            
            if reward <= 0 {
                panic!("No rewards to claim");
            }
            
            // Token::Client yerine doğrudan depolama işlemlerini kullan
            spend_balance(&e, e.current_contract_address(), reward);
            receive_balance(&e, user.clone(), reward);
            
            // Son çekim zamanını güncelle
            stake_info.last_claim_ledger = e.ledger().sequence();
            stakes.set(user.clone(), stake_info);
            
            // Güncellenmiş bilgileri kaydet
            e.storage().instance().set(&STAKES_KEY, &stakes);
            
            // TTL süresini uzat
            e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
            
            // Ödül çekme olayını yayınla
            emit_event(&e, "claim_reward", &user, reward);
            
            return reward;
        } else {
            panic!("No stake found for user");
        }
    }
    
    // Hesaplanabilir ödülü görüntüleme fonksiyonu (view fonksiyonu)
    pub fn get_pending_rewards(e: Env, user: Address) -> i128 {
        // Havuz ve stake bilgilerini al
        let pool_info: PoolInfo = e.storage().instance().get(&POOL_INFO_KEY).unwrap();
        let stakes: Map<Address, StakeInfo> = e.storage().instance().get(&STAKES_KEY).unwrap();
        
        // Kullanıcının stake bilgisini kontrol et
        if let Some(stake_info) = stakes.get(user.clone()) {
            // Bekleyen ödülü hesapla
            return Self::calculate_reward(&e, &user, &stake_info, &pool_info);
        } else {
            return 0;
        }
    }
    
    // Stake çekme fonksiyonu
pub fn unstake(e: Env, user: Address, amount: i128) -> i128 {
    // Kullanıcının yetkilendirmesini kontrol et
    user.require_auth();
    
    // Negatif miktar kontrolü
    if amount <= 0 {
        panic!("Unstake amount must be positive");
    }
    
    // Havuz ve stake bilgilerini al
    let mut pool_info: PoolInfo = e.storage().instance().get(&POOL_INFO_KEY).unwrap();
    let mut stakes: Map<Address, StakeInfo> = e.storage().instance().get(&STAKES_KEY).unwrap();
    
    // Kullanıcının stake bilgisini kontrol et
    if let Some(mut stake_info) = stakes.get(user.clone()) {
        // Miktarın kullanıcının toplam stake'inden az olduğunu kontrol et
        if amount > stake_info.amount {
            panic!("Unstake amount exceeds staked amount");
        }
        
        // Minimum stake süresinin geçip geçmediğini kontrol et
        let current_ledger = e.ledger().sequence();
        if current_ledger - stake_info.since_ledger < pool_info.min_stake_duration {
            panic!("Minimum stake duration not met");
        }
        
        // Önce bekleyen ödülleri hesapla
        let reward = Self::calculate_reward(&e, &user, &stake_info, &pool_info);
        
        // Varsa ödülleri gönder (token::Client yerine depolama işlemleri ile)
        if reward > 0 {
            spend_balance(&e, e.current_contract_address(), reward);
            receive_balance(&e, user.clone(), reward);
            
            // Ödül çekme olayını yayınla
            emit_event(&e, "claim_reward", &user, reward);
        }
        
        // Kullanıcıya tokenlarını geri gönder (token::Client yerine depolama işlemleri ile)
        spend_balance(&e, e.current_contract_address(), amount);
        receive_balance(&e, user.clone(), amount);
        
        // Stake miktarını ve toplam stake miktarını güncelle
        stake_info.amount -= amount;
        pool_info.total_staked -= amount;
        
        // Eğer kalan miktar 0 ise kaydı sil, değilse güncelle
        if stake_info.amount == 0 {
            stakes.remove(user.clone());
        } else {
            stake_info.last_claim_ledger = current_ledger;
            stakes.set(user.clone(), stake_info);
        }
        
        // Güncellenmiş bilgileri kaydet
        e.storage().instance().set(&POOL_INFO_KEY, &pool_info);
        e.storage().instance().set(&STAKES_KEY, &stakes);
        
        // TTL süresini uzat
       
        e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        // Unstake olayını yayınla
        emit_event(&e, "unstake", &user, amount);
        
        return amount;
    } else {
        panic!("No stake found for user");
    }
}
    
    // Kullanıcının stake bilgisini görüntüleme fonksiyonu
    pub fn get_stake_info(e: Env, user: Address) -> StakeInfo {
        let stakes: Map<Address, StakeInfo> = e.storage().instance().get(&STAKES_KEY).unwrap();
        
        if let Some(stake_info) = stakes.get(user) {
            return stake_info;
        } else {
            panic!("No stake found for user");
        }
    }
    
    // Havuz bilgilerini görüntüleme fonksiyonu
    pub fn get_pool_info(e: Env) -> PoolInfo {
        e.storage().instance().get(&POOL_INFO_KEY).unwrap()
    }
    
    // Acil durum fonksiyonu: Admin tüm ödül tokenlarını çekebilir (sadece acil durumlar için)
    pub fn emergency_withdraw_rewards(e: Env) -> i128 {
        // Admin kontrolü
        let admin: Address = e.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Havuz bilgilerini al
        let pool_info: PoolInfo = e.storage().instance().get(&POOL_INFO_KEY).unwrap();
        
        // Kontrattaki ödül token bakiyesini al - doğrudan depolama fonksiyonu kullan
        let balance = read_balance(&e, e.current_contract_address());
        
        // Tüm bakiyeyi admin'e transfer et
        if balance > 0 {
            // Token::Client yerine doğrudan depolama işlemlerini kullan
            spend_balance(&e, e.current_contract_address(), balance);
            receive_balance(&e, admin.clone(), balance);
            
            // Acil çekim olayını yayınla
            emit_event(&e, "emergency_withdraw", &admin, balance);
        }
        
        return balance;
    }
}
  

#[contractimpl]
impl token::Interface for Token {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Göndericinin hesabı dondurulmuş mu kontrol et
        if is_account_frozen(&e, &from) {
            panic!("Hesap dondurulmuş ve token transfer edilemez");
        }

        // Transferi gerçekleştir
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Göndericinin hesabı dondurulmuş mu kontrol et
        if is_account_frozen(&e, &from) {
            panic!("Hesap dondurulmuş ve token transfer edilemez");
        }

         // Transferi gerçekleştir
        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Göndericinin hesabı dondurulmuş mu kontrol et
        if is_account_frozen(&e, &from) {
            panic!("Hesap dondurulmuş ve token yakılamaz");
        }

        // Yakma işlemini gerçekleştir
        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

         // Göndericinin hesabı dondurulmuş mu kontrol et
         if is_account_frozen(&e, &from) {
            panic!("Hesap dondurulmuş ve token yakılamaz");
        }

        // Yakma işlemini gerçekleştir
        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount)
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}