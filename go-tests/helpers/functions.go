package helpers

import (
	"github.com/stakingagency/sa-mx-sdk-go/data"
	"github.com/stakingagency/sa-mx-sdk-go/utils"
)

func delegate(amount float64) error {
	return salsa.Delegate(pk, amount, 100000000, nil, utils.AutoNonce, false)
}

func delegateCustodial(amount float64) error {
	return salsa.Delegate(pk, amount, 100000000, nil, utils.AutoNonce, true)
}

func addReserve(amount float64) error {
	return salsa.AddReserve(pk, amount, 100000000, nil, utils.AutoNonce)
}

func removeReserve(amount string) error {
	return salsa.RemoveReserve(pk, 0, 100000000, nil, utils.AutoNonce, str2big(amount))
}

func undelegateNow(amount float64) error {
	return salsa.UnDelegateNow(pk, amount, 100000000, &data.ESDT{Ticker: tokenID, Decimals: 18}, utils.AutoNonce, str2big("0"), str2big("0"))
}

func undelegateNowCustodial(amount string) error {
	return salsa.UnDelegateNowCustodial(pk, 0, 100000000, nil, utils.AutoNonce, str2big("0"), str2big(amount))
}

func takeLpProfit() error {
	return salsa.TakeLpProfit(pk, 0, 100000000, nil, utils.AutoNonce)
}
