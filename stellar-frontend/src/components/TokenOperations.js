'use client';
import { useState } from 'react';
import { useSorobanReact } from '@soroban-react/core';
import {
    Contract,
    nativeToScVal,
    Address
} from '@stellar/stellar-sdk';
import ConnectWallet from './ConnectWallet';
import TransactionHistory from './TransactionHistory';
import './TokenDashboard.css';

// İkon komponentleri
const TransferIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="menu-icon">
        <path d="M4.5 3.75a3 3 0 00-3 3v.75h21v-.75a3 3 0 00-3-3h-15z" />
        <path fillRule="evenodd" d="M22.5 9.75h-21v7.5a3 3 0 003 3h15a3 3 0 003-3v-7.5zm-18 3.75a.75.75 0 01.75-.75h6a.75.75 0 010 1.5h-6a.75.75 0 01-.75-.75zm.75 2.25a.75.75 0 000 1.5h3a.75.75 0 000-1.5h-3z" clipRule="evenodd" />
    </svg>
);

const BurnIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="menu-icon">
        <path fillRule="evenodd" d="M12.963 2.286a.75.75 0 00-1.071-.136 9.742 9.742 0 00-3.539 6.177A7.547 7.547 0 016.648 6.61a.75.75 0 00-1.152.082A9 9 0 1015.68 4.534a7.46 7.46 0 01-2.717-2.248zM15.75 14.25a3.75 3.75 0 11-7.313-1.172c.628.465 1.35.81 2.133 1a5.99 5.99 0 011.925-3.545 3.75 3.75 0 013.255 3.717z" clipRule="evenodd" />
    </svg>
);

const StakeIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="menu-icon">
        <path d="M21 6.375c0 2.692-4.03 4.875-9 4.875S3 9.067 3 6.375 7.03 1.5 12 1.5s9 2.183 9 4.875z" />
        <path d="M12 12.75c2.685 0 5.19-.586 7.078-1.609a8.283 8.283 0 001.897-1.384c.016.121.025.244.025.368C21 12.817 16.97 15 12 15s-9-2.183-9-4.875c0-.124.009-.247.025-.368a8.285 8.285 0 001.897 1.384C6.809 12.164 9.315 12.75 12 12.75z" />
        <path d="M12 16.5c2.685 0 5.19-.586 7.078-1.609a8.282 8.282 0 001.897-1.384c.016.121.025.244.025.368 0 2.692-4.03 4.875-9 4.875s-9-2.183-9-4.875c0-.124.009-.247.025-.368a8.284 8.284 0 001.897 1.384C6.809 15.914 9.315 16.5 12 16.5z" />
        <path d="M12 20.25c2.685 0 5.19-.586 7.078-1.609a8.282 8.282 0 001.897-1.384c.016.121.025.244.025.368 0 2.692-4.03 4.875-9 4.875s-9-2.183-9-4.875c0-.124.009-.247.025-.368a8.284 8.284 0 001.897 1.384C6.809 19.664 9.315 20.25 12 20.25z" />
    </svg>
);

const VestingIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="menu-icon">
        <path fillRule="evenodd" d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 2.25zM12.75 6a.75.75 0 00-1.5 0v6c0 .414.336.75.75.75h4.5a.75.75 0 000-1.5h-3.75V6z" clipRule="evenodd" />
    </svg>
);

const HistoryIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="menu-icon">
        <path fillRule="evenodd" d="M7.502 6h7.128A3.375 3.375 0 0118 9.375v9.375a3 3 0 003-3V6.108c0-1.505-1.125-2.811-2.664-2.94a48.972 48.972 0 00-.673-.05A3 3 0 0015 1.5h-1.5a3 3 0 00-2.663 1.618c-.225.015-.45.032-.673.05C8.662 3.295 7.554 4.542 7.502 6zM13.5 3A1.5 1.5 0 0012 4.5h4.5A1.5 1.5 0 0015 3h-1.5z" clipRule="evenodd" />
        <path fillRule="evenodd" d="M3 9.375C3 8.339 3.84 7.5 4.875 7.5h9.75c1.036 0 1.875.84 1.875 1.875v11.25c0 1.035-.84 1.875-1.875 1.875h-9.75A1.875 1.875 0 013 20.625V9.375zM6 12a.75.75 0 01.75-.75h.008a.75.75 0 01.75.75v.008a.75.75 0 01-.75.75H6.75a.75.75 0 01-.75-.75V12zm2.25 0a.75.75 0 01.75-.75h3.75a.75.75 0 010 1.5H9a.75.75 0 01-.75-.75zM6 15a.75.75 0 01.75-.75h.008a.75.75 0 01.75.75v.008a.75.75 0 01-.75.75H6.75a.75.75 0 01-.75-.75V15zm2.25 0a.75.75 0 01.75-.75h3.75a.75.75 0 010 1.5H9a.75.75 0 01-.75-.75zM6 18a.75.75 0 01.75-.75h.008a.75.75 0 01.75.75v.008a.75.75 0 01-.75.75H6.75a.75.75 0 01-.75-.75V18zm2.25 0a.75.75 0 01.75-.75h3.75a.75.75 0 010 1.5H9a.75.75 0 01-.75-.75z" clipRule="evenodd" />
    </svg>
);

const FreezeIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="menu-icon">
        <path fillRule="evenodd" d="M11.584 2.376a.75.75 0 01.832 0l9 6a.75.75 0 11-.832 1.248L12 3.901 3.416 9.624a.75.75 0 01-.832-1.248l9-6zM20.25 10.332v9.918H21a.75.75 0 010 1.5H3a.75.75 0 010-1.5h.75v-9.918a.75.75 0 01.634-.74A49.109 49.109 0 0112 9c2.59 0 5.134.202 7.616.592a.75.75 0 01.634.74zm-7.5 2.418a.75.75 0 00-1.5 0v6.75a.75.75 0 001.5 0v-6.75zm3-.75a.75.75 0 01.75.75v6.75a.75.75 0 01-1.5 0v-6.75a.75.75 0 01.75-.75zM9 12.75a.75.75 0 00-1.5 0v6.75a.75.75 0 001.5 0v-6.75z" clipRule="evenodd" />
        <path d="M12 7.875a1.125 1.125 0 100-2.25 1.125 1.125 0 000 2.25z" />
    </svg>
);

const WalletIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="wallet-icon">
        <path d="M2.273 5.625A4.483 4.483 0 015.25 4.5h13.5c1.141 0 2.183.425 2.977 1.125A3 3 0 0018.75 3H5.25a3 3 0 00-2.977 2.625zM2.273 8.625A4.483 4.483 0 015.25 7.5h13.5c1.141 0 2.183.425 2.977 1.125A3 3 0 0018.75 6H5.25a3 3 0 00-2.977 2.625zM5.25 9a3 3 0 00-3 3v6a3 3 0 003 3h13.5a3 3 0 003-3v-6a3 3 0 00-3-3H15a.75.75 0 00-.75.75 2.25 2.25 0 01-4.5 0A.75.75 0 009 9H5.25z" />
    </svg>
);

export default function TokenDashboard({ contractId }) {
    const { address, server, activeChain } = useSorobanReact();
    const [activeTab, setActiveTab] = useState('transfer');
    const [isSubmitting, setIsSubmitting] = useState(false);

    // Statik bakiye değeri - dinamik sorgulama yerine sabit değer
    const balance = '1000000'; // İstenen statik bakiye değeri

    // Transfer state
    const [transferAmount, setTransferAmount] = useState('');
    const [transferRecipient, setTransferRecipient] = useState('');

    // Burn state
    const [burnAmount, setBurnAmount] = useState('');

    // Stake state
    const [stakeAmount, setStakeAmount] = useState('');
    const [stakeDuration, setStakeDuration] = useState('30');

    // Vesting state
    const [vestingAmount, setVestingAmount] = useState('');
    const [vestingRecipient, setVestingRecipient] = useState('');
    const [vestingDuration, setVestingDuration] = useState('90');
    const [vestingCliff, setVestingCliff] = useState('30');

    // Freeze/Unfreeze state
    const [freezeAccountAddress, setFreezeAccountAddress] = useState('');
    const [unfreezeAccountAddress, setUnfreezeAccountAddress] = useState('');

    // Transfer işlemi
    const handleTransfer = async (e) => {
        e.preventDefault();
        if (!address || !transferRecipient || !transferAmount || isSubmitting) return;
        setIsSubmitting(true);

        try {
            const contract = new Contract(contractId);
            console.log("Transfer başlatılıyor");

            const result = await contract.call(
                server,
                'transfer',
                {
                    args: [
                        new Address(address).toScVal(),
                        new Address(transferRecipient).toScVal(),
                        nativeToScVal(parseInt(transferAmount), { type: 'i128' })
                    ],
                    fee: '1000000'
                }
            );

            console.log('Transfer sonucu:', result);
            alert('Transfer işlemi gönderildi!');
            setTransferAmount('');
            setTransferRecipient('');

            // Bakiye artık statik olduğu için güncelleme adımı kaldırıldı
            await new Promise(resolve => setTimeout(resolve, 2000));
        } catch (error) {
            console.error('Transfer hatası:', error);
            alert(`Transfer başarısız: ${error.message}`);
        } finally {
            setIsSubmitting(false);
        }
    };

    // Burn işlemi
    const handleBurn = async (e) => {
        e.preventDefault();
        if (!address || !burnAmount || isSubmitting) return;
        setIsSubmitting(true);

        try {
            const contract = new Contract(contractId);

            const result = await contract.call(
                server,
                'burn',
                {
                    args: [
                        new Address(address).toScVal(),
                        nativeToScVal(parseInt(burnAmount), { type: 'i128' })
                    ],
                    fee: '1000000'
                }
            );

            console.log('Burn sonucu:', result);
            alert('Token yakma işlemi gönderildi!');
            setBurnAmount('');

            // Bakiye artık statik olduğu için güncelleme adımı kaldırıldı
            await new Promise(resolve => setTimeout(resolve, 2000));
        } catch (error) {
            console.error('Burn hatası:', error);
            alert(`Token yakma başarısız: ${error.message}`);
        } finally {
            setIsSubmitting(false);
        }
    };

    // Stake işlemi
    const handleStake = async (e) => {
        e.preventDefault();
        if (!address || !stakeAmount || !stakeDuration || isSubmitting) return;
        setIsSubmitting(true);

        try {
            const contract = new Contract(contractId);

            const result = await contract.call(
                server,
                'stake',
                {
                    args: [
                        new Address(address).toScVal(),
                        nativeToScVal(parseInt(stakeAmount), { type: 'i128' }),
                        nativeToScVal(parseInt(stakeDuration), { type: 'u32' })
                    ],
                    fee: '1000000'
                }
            );

            console.log('Stake sonucu:', result);
            alert('Stake işlemi gönderildi!');
            setStakeAmount('');

            // Bakiye artık statik olduğu için güncelleme adımı kaldırıldı
            await new Promise(resolve => setTimeout(resolve, 2000));
        } catch (error) {
            console.error('Stake hatası:', error);
            alert(`Stake işlemi başarısız: ${error.message}`);
        } finally {
            setIsSubmitting(false);
        }
    };

    // Vesting işlemi
    const handleVesting = async (e) => {
        e.preventDefault();
        if (!address || !vestingRecipient || !vestingAmount || !vestingDuration || !vestingCliff || isSubmitting) return;
        setIsSubmitting(true);

        try {
            const contract = new Contract(contractId);

            const result = await contract.call(
                server,
                'create_vesting',
                {
                    args: [
                        new Address(address).toScVal(),
                        new Address(vestingRecipient).toScVal(),
                        nativeToScVal(parseInt(vestingAmount), { type: 'i128' }),
                        nativeToScVal(parseInt(vestingDuration), { type: 'u32' }),
                        nativeToScVal(parseInt(vestingCliff), { type: 'u32' })
                    ],
                    fee: '1000000'
                }
            );

            console.log('Vesting sonucu:', result);
            alert('Vesting işlemi gönderildi!');
            setVestingAmount('');
            setVestingRecipient('');

            // Bakiye artık statik olduğu için güncelleme adımı kaldırıldı
            await new Promise(resolve => setTimeout(resolve, 2000));
        } catch (error) {
            console.error('Vesting hatası:', error);
            alert(`Vesting işlemi başarısız: ${error.message}`);
        } finally {
            setIsSubmitting(false);
        }
    };

    // Freeze Account işlemi
    const handleFreezeAccount = async (e) => {
        e.preventDefault();
        if (!address || !freezeAccountAddress || isSubmitting) return;
        setIsSubmitting(true);

        try {
            const contract = new Contract(contractId);

            const result = await contract.call(
                server,
                'freeze_account',
                {
                    args: [
                        new Address(freezeAccountAddress).toScVal()
                    ],
                    fee: '1000000'
                }
            );

            console.log('Freeze account sonucu:', result);
            alert('Hesap dondurma işlemi gönderildi!');
            setFreezeAccountAddress('');

            await new Promise(resolve => setTimeout(resolve, 2000));
        } catch (error) {
            console.error('Freeze account hatası:', error);
            alert(`Hesap dondurma işlemi başarısız: ${error.message}`);
        } finally {
            setIsSubmitting(false);
        }
    };

    // Unfreeze Account işlemi
    const handleUnfreezeAccount = async (e) => {
        e.preventDefault();
        if (!address || !unfreezeAccountAddress || isSubmitting) return;
        setIsSubmitting(true);

        try {
            const contract = new Contract(contractId);

            const result = await contract.call(
                server,
                'unfreeze_account',
                {
                    args: [
                        new Address(unfreezeAccountAddress).toScVal()
                    ],
                    fee: '1000000'
                }
            );

            console.log('Unfreeze account sonucu:', result);
            alert('Hesap çözme işlemi gönderildi!');
            setUnfreezeAccountAddress('');

            await new Promise(resolve => setTimeout(resolve, 2000));
        } catch (error) {
            console.error('Unfreeze account hatası:', error);
            alert(`Hesap çözme işlemi başarısız: ${error.message}`);
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <div className="token-dashboard">
            <div className="dashboard-header">
                <h1 className="dashboard-title">Token Yönetim Paneli</h1>
                <div className="wallet-connect-button">
                    <WalletIcon />
                    <ConnectWallet />
                </div>
            </div>

            {address ? (
                <div className="dashboard-content">
                    {/* Sol sidebar - Bakiye ve İşlem Menüsü */}
                    <div className="dashboard-sidebar">
                        <div className="balance-card">
                            <h2 className="balance-title">Token Bakiyeniz</h2>
                            <div className="balance-amount">
                                {balance} <span className="balance-currency">TKN</span>
                            </div>
                            {/* Refresh butonu kaldırıldı - bakiye statik olduğundan yenileme yapmıyoruz */}
                        </div>

                        <div className="menu-list">
                            <div className="menu-item">
                                <button
                                    onClick={() => setActiveTab('transfer')}
                                    className={`menu-button ${activeTab === 'transfer' ? 'active transfer' : ''}`}
                                >
                                    <TransferIcon />
                                    Transfer
                                </button>
                            </div>
                            <div className="menu-item">
                                <button
                                    onClick={() => setActiveTab('burn')}
                                    className={`menu-button ${activeTab === 'burn' ? 'active burn' : ''}`}
                                >
                                    <BurnIcon />
                                    Token Yak
                                </button>
                            </div>
                            <div className="menu-item">
                                <button
                                    onClick={() => setActiveTab('stake')}
                                    className={`menu-button ${activeTab === 'stake' ? 'active stake' : ''}`}
                                >
                                    <StakeIcon />
                                    Stake Et
                                </button>
                            </div>
                            <div className="menu-item">
                                <button
                                    onClick={() => setActiveTab('vesting')}
                                    className={`menu-button ${activeTab === 'vesting' ? 'active vesting' : ''}`}
                                >
                                    <VestingIcon />
                                    Vesting Oluştur
                                </button>
                            </div>
                            <div className="menu-item">
                                <button
                                    onClick={() => setActiveTab('freeze')}
                                    className={`menu-button ${activeTab === 'freeze' ? 'active freeze' : ''}`}
                                >
                                    <FreezeIcon />
                                    Hesap Dondur/Çöz
                                </button>
                            </div>
                            <div className="menu-item">
                                <button
                                    onClick={() => setActiveTab('history')}
                                    className={`menu-button ${activeTab === 'history' ? 'active history' : ''}`}
                                >
                                    <HistoryIcon />
                                    İşlem Geçmişi
                                </button>
                            </div>
                        </div>
                    </div>

                    {/* Ana içerik alanı */}
                    <div className="dashboard-main">
                        {/* Transfer Tab */}
                        {activeTab === 'transfer' && (
                            <div>
                                <h2 className="tab-title transfer-title">Token Transfer</h2>
                                <form onSubmit={handleTransfer} className="form-content">
                                    <div className="form-group">
                                        <label className="input-label">Alıcı Adresi</label>
                                        <input
                                            type="text"
                                            placeholder="GABC123..."
                                            className="input-field"
                                            value={transferRecipient}
                                            onChange={(e) => setTransferRecipient(e.target.value)}
                                            required
                                        />
                                        <p className="input-help">Alıcının Stellar adresini girin</p>
                                    </div>

                                    <div className="form-group">
                                        <label className="input-label">Miktar</label>
                                        <div className="input-addon">
                                            <input
                                                type="number"
                                                placeholder="0"
                                                className="input-field"
                                                value={transferAmount}
                                                onChange={(e) => setTransferAmount(e.target.value)}
                                                min="1"
                                                required
                                            />
                                            <span className="input-addon-text">TKN</span>
                                        </div>
                                        <p className="input-help">Göndermek istediğiniz token miktarı</p>
                                    </div>

                                    <div className="info-box info">
                                        <div className="info-box-title">Bilgi</div>
                                        <p>Transfer işlemi, tokenlarınızı başka bir adrese göndermenizi sağlar.</p>
                                    </div>

                                    <button
                                        type="submit"
                                        disabled={isSubmitting || !transferAmount || !transferRecipient}
                                        className={`button button-blue`}
                                    >
                                        {isSubmitting ? 'İşleniyor...' : 'Transfer Et'}
                                    </button>
                                </form>
                            </div>
                        )}

                        {/* Burn Tab */}
                        {activeTab === 'burn' && (
                            <div>
                                <h2 className="tab-title burn-title">Token Yakma</h2>
                                <form onSubmit={handleBurn} className="form-content">
                                    <div className="form-group">
                                        <label className="input-label">Miktar</label>
                                        <div className="input-addon">
                                            <input
                                                type="number"
                                                placeholder="0"
                                                className="input-field"
                                                value={burnAmount}
                                                onChange={(e) => setBurnAmount(e.target.value)}
                                                min="1"
                                                required
                                            />
                                            <span className="input-addon-text">TKN</span>
                                        </div>
                                        <p className="input-help">Yakmak istediğiniz token miktarı</p>
                                    </div>

                                    <div className="info-box warning">
                                        <div className="info-box-title">Uyarı</div>
                                        <p>Token yakma işlemi geri alınamaz. Yakılan tokenlar kalıcı olarak dolaşımdan çıkarılır.</p>
                                    </div>

                                    <button
                                        type="submit"
                                        disabled={isSubmitting || !burnAmount}
                                        className={`button button-red`}
                                    >
                                        {isSubmitting ? 'İşleniyor...' : 'Tokenleri Yak'}
                                    </button>
                                </form>
                            </div>
                        )}

                        {/* Stake Tab */}
                        {activeTab === 'stake' && (
                            <div>
                                <h2 className="tab-title stake-title">Token Stake Et</h2>
                                <form onSubmit={handleStake} className="form-content">
                                    <div className="form-group">
                                        <label className="input-label">Miktar</label>
                                        <div className="input-addon">
                                            <input
                                                type="number"
                                                placeholder="0"
                                                className="input-field"
                                                value={stakeAmount}
                                                onChange={(e) => setStakeAmount(e.target.value)}
                                                min="1"
                                                required
                                            />
                                            <span className="input-addon-text">TKN</span>
                                        </div>
                                        <p className="input-help">Stake etmek istediğiniz token miktarı</p>
                                    </div>

                                    <div className="form-group">
                                        <label className="input-label">Stake Süresi</label>
                                        <select
                                            className="select-field"
                                            value={stakeDuration}
                                            onChange={(e) => setStakeDuration(e.target.value)}
                                            required
                                        >
                                            <option value="30">30 Gün</option>
                                            <option value="90">90 Gün</option>
                                            <option value="180">180 Gün</option>
                                            <option value="365">365 Gün</option>
                                        </select>
                                        <p className="input-help">Tokenlerin kilitli kalacağı süre</p>
                                    </div>

                                    <div className="info-box success">
                                        <div className="info-box-title">Bilgi</div>
                                        <p>Stake ettiğiniz tokenlar belirtilen süre boyunca kilitlenir. Bu süre içinde %8 yıllık getiri kazanırsınız.</p>
                                    </div>

                                    <button
                                        type="submit"
                                        disabled={isSubmitting || !stakeAmount}
                                        className={`button button-green`}
                                    >
                                        {isSubmitting ? 'İşleniyor...' : 'Stake Et'}
                                    </button>
                                </form>
                            </div>
                        )}

                        {/* Vesting Tab */}
                        {activeTab === 'vesting' && (
                            <div>
                                <h2 className="tab-title vesting-title">Vesting Planı Oluştur</h2>
                                <form onSubmit={handleVesting} className="form-content">
                                    <div className="form-group">
                                        <label className="input-label">Alıcı Adresi</label>
                                        <input
                                            type="text"
                                            placeholder="GABC123..."
                                            className="input-field"
                                            value={vestingRecipient}
                                            onChange={(e) => setVestingRecipient(e.target.value)}
                                            required
                                        />
                                        <p className="input-help">Vesting planı için alıcı adresi</p>
                                    </div>

                                    <div className="form-group">
                                        <label className="input-label">Toplam Miktar</label>
                                        <div className="input-addon">
                                            <input
                                                type="number"
                                                placeholder="0"
                                                className="input-field"
                                                value={vestingAmount}
                                                onChange={(e) => setVestingAmount(e.target.value)}
                                                min="1"
                                                required
                                            />
                                            <span className="input-addon-text">TKN</span>
                                        </div>
                                        <p className="input-help">Vesting planı için toplam token miktarı</p>
                                    </div>

                                    <div className="form-group">
                                        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '1rem' }}>
                                            <div>
                                                <label className="input-label">Vesting Süresi</label>
                                                <select
                                                    className="select-field"
                                                    value={vestingDuration}
                                                    onChange={(e) => setVestingDuration(e.target.value)}
                                                    required
                                                >
                                                    <option value="90">90 Gün</option>
                                                    <option value="180">180 Gün</option>
                                                    <option value="365">365 Gün</option>
                                                    <option value="730">730 Gün</option>
                                                </select>
                                                <p className="input-help">Toplam vesting süresi</p>
                                            </div>

                                            <div>
                                                <label className="input-label">Cliff Süresi</label>
                                                <select
                                                    className="select-field"
                                                    value={vestingCliff}
                                                    onChange={(e) => setVestingCliff(e.target.value)}
                                                    required
                                                >
                                                    <option value="0">0 Gün (Yok)</option>
                                                    <option value="30">30 Gün</option>
                                                    <option value="60">60 Gün</option>
                                                    <option value="90">90 Gün</option>
                                                </select>
                                                <p className="input-help">İlk token serbest kalma süresi</p>
                                            </div>
                                        </div>
                                    </div>

                                    <div className="info-box purple">
                                        <div className="info-box-title">Bilgi</div>
                                        <p>Vesting planı, belirtilen süre boyunca tokenlerin kademeli olarak serbest bırakılmasını sağlar.
                                            Cliff süresi dolana kadar hiçbir token serbest bırakılmaz, sonrasında kalan süre boyunca tokenlar doğrusal olarak serbest kalır.</p>
                                    </div>

                                    <button
                                        type="submit"
                                        disabled={isSubmitting || !vestingAmount || !vestingRecipient}
                                        className={`button button-purple`}
                                    >
                                        {isSubmitting ? 'İşleniyor...' : 'Vesting Planı Oluştur'}
                                    </button>
                                </form>
                            </div>
                        )}

                        {/* Freeze/Unfreeze Tab */}
                        {activeTab === 'freeze' && (
                            <div>
                                <h2 className="tab-title freeze-title">Hesap Dondurma / Çözme</h2>

                                <div className="freeze-container">
                                    <div className="freeze-section">
                                        <h3 className="section-title">Hesap Dondur</h3>
                                        <form onSubmit={handleFreezeAccount} className="form-content">
                                            <div className="form-group">
                                                <label className="input-label">Hesap Adresi</label>
                                                <input
                                                    type="text"
                                                    placeholder="GABC123..."
                                                    className="input-field"
                                                    value={freezeAccountAddress}
                                                    onChange={(e) => setFreezeAccountAddress(e.target.value)}
                                                    required
                                                />
                                                <p className="input-help">Dondurmak istediğiniz hesabın Stellar adresi</p>
                                            </div>

                                            <div className="info-box warning">
                                                <div className="info-box-title">Uyarı</div>
                                                <p>Dondurulmuş hesaplar transfer işlemi gerçekleştiremez. Sadece yönetici bu işlemi gerçekleştirebilir.</p>
                                            </div>

                                            <button
                                                type="submit"
                                                disabled={isSubmitting || !freezeAccountAddress}
                                                className="button button-red"
                                            >
                                                {isSubmitting ? 'İşleniyor...' : 'Hesabı Dondur'}
                                            </button>
                                        </form>
                                    </div>

                                    <div className="divider"></div>

                                    <div className="unfreeze-section">
                                        <h3 className="section-title">Hesap Çöz</h3>
                                        <form onSubmit={handleUnfreezeAccount} className="form-content">
                                            <div className="form-group">
                                                <label className="input-label">Hesap Adresi</label>
                                                <input
                                                    type="text"
                                                    placeholder="GABC123..."
                                                    className="input-field"
                                                    value={unfreezeAccountAddress}
                                                    onChange={(e) => setUnfreezeAccountAddress(e.target.value)}
                                                    required
                                                />
                                                <p className="input-help">Çözmek istediğiniz hesabın Stellar adresi</p>
                                            </div>

                                            <div className="info-box info">
                                                <div className="info-box-title">Bilgi</div>
                                                <p>Hesabı çözdüğünüzde, kullanıcı tekrar token transferi yapabilir hale gelecektir.</p>
                                            </div>

                                            <button
                                                type="submit"
                                                disabled={isSubmitting || !unfreezeAccountAddress}
                                                className="button button-blue"
                                            >
                                                {isSubmitting ? 'İşleniyor...' : 'Hesabı Çöz'}
                                            </button>
                                        </form>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* İşlem Geçmişi Tab */}
                        {activeTab === 'history' && (
                            <div>
                                <h2 className="tab-title history-title">İşlem Geçmişi</h2>
                                <TransactionHistory />
                            </div>
                        )}
                    </div>
                </div>
            ) : (
                <div className="wallet-card">
                    <h2 className="wallet-title">Token İşlemlerine Başlayın</h2>
                    <p className="wallet-description">Token işlemlerini gerçekleştirmek için Freighter cüzdanınızı bağlayın.</p>
                    <ConnectWallet />
                </div>
            )}
        </div>
    );
}