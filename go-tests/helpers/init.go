package helpers

import (
	"encoding/hex"

	"github.com/multiversx/mx-chain-core-go/core/pubkeyConverter"
	logger "github.com/multiversx/mx-chain-logger-go"
	"github.com/multiversx/mx-sdk-go/interactors"
	"github.com/stakingagency/sa-mx-sdk-go/accounts"
	"github.com/stakingagency/sa-mx-sdk-go/utils"
	"github.com/stakingagency/salsa-sc/go-tests/oneDex"
	"github.com/stakingagency/salsa-sc/go-tests/salsaContract"
	"github.com/stakingagency/salsa-sc/go-tests/xExchange"
)

func InitSC() error {
	var err error
	salsa, err = salsaContract.NewSalsaContract(scAddress, proxyAddress, indexAddress)
	if err != nil {
		return err
	}

	onedex, _ = oneDex.NewOneDex(onedexSC, proxyAddress, indexAddress)
	xexchange, _ = xExchange.NewPair(xexchangeSC, proxyAddress, indexAddress)

	w := interactors.NewWallet()
	pk, err = w.LoadPrivateKeyFromPemFile(walletFile)
	if err != nil {
		return err
	}

	scAccount, err = accounts.NewAccount(scAddress, salsa.GetNetworkManager(), utils.NoRefresh)
	if err != nil {
		return err
	}

	return configureSC()
}

func configureSC() error {
	var err error
	tokenID, err = readStringKey("liquid_token_id")
	if err != nil {
		return err
	}

	if tokenID == "" {
		err = salsa.RegisterLiquidToken(pk, 0.05, 35000000, nil, utils.AutoNonce, "TEST", "TEST", 18)
		if err != nil {
			return err
		}

		tokenID, err = readStringKey("liquid_token_id")
		if err != nil {
			return err
		}
	}

	if !keyExists("undelegate_now_fee") {
		err = salsa.SetUndelegateNowFee(pk, 0, 10000000, nil, utils.AutoNonce, 200)
		if err != nil {
			return err
		}
	}

	if !keyExists("provider_address") {
		provider, _ := hex.DecodeString(providerAddress)
		err = salsa.SetProviderAddress(pk, 0, 10000000, nil, utils.AutoNonce, salsaContract.Address(provider))
		if err != nil {
			return err
		}
	}

	if !keyExists("unbond_period") {
		err = salsa.SetUnbondPeriod(pk, 0, 10000000, nil, utils.AutoNonce, 1)
		if err != nil {
			return err
		}
	}

	if !keyExists("state") {
		err = salsa.SetStateActive(pk, 0, 10000000, nil, utils.AutoNonce)
		if err != nil {
			return err
		}
	}

	return nil
}

func StartArbitrage() error {
	if !keyExists("wrap_sc") {
		address, _ := hex.DecodeString(wrapSC)
		err := salsa.SetWrapSC(pk, 0, 10000000, nil, utils.AutoNonce, address)
		if err != nil {
			return err
		}
	}

	conv, _ := pubkeyConverter.NewBech32PubkeyConverter(32, logger.GetOrCreate("init"))
	oSCh, _ := conv.Decode(onedexSC)

	if !keyExists("onedex_sc") {
		err := salsa.SetOnedexSC(pk, 0, 10000000, nil, utils.AutoNonce, oSCh)
		if err != nil {
			return err
		}
	}

	xSCh, _ := conv.Decode(xexchangeSC)
	if !keyExists("xexchange_sc") {
		err := salsa.SetXexchangeSC(pk, 0, 10000000, nil, utils.AutoNonce, xSCh)
		if err != nil {
			return err
		}
	}

	if !keyExists("onedex_pair_id") {
		err := salsa.SetOnedexPairId(pk, 0, 10000000, nil, utils.AutoNonce, onedexPairID)
		if err != nil {
			return err
		}
	}

	if !keyExists("onedex_arbitrage") {
		err := salsa.SetOnedexArbitrageActive(pk, 0, 20000000, nil, utils.AutoNonce)
		if err != nil {
			return err
		}
	}

	if !keyExists("xexchange_arbitrage") {
		err := salsa.SetXexchangeArbitrageActive(pk, 0, 20000000, nil, utils.AutoNonce)
		if err != nil {
			return err
		}
	}

	if !keyExists("arbitrage") {
		err := salsa.SetArbitrageActive(pk, 0, 20000000, nil, utils.AutoNonce)
		if err != nil {
			return err
		}
	}

	return nil
}
