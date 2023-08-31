package helpers

import (
	"fmt"

	"github.com/stakingagency/sa-mx-sdk-go/accounts"
	"github.com/stakingagency/sa-mx-sdk-go/utils"
)

func GetLpInfo() error {
	sc, _ := accounts.NewAccount(scAddress, salsa.GetNetworkManager(), utils.NoRefresh)
	balances, err := sc.GetTokensBalances()
	if err == nil {
		fmt.Printf("XLP %.4f, ONELP %.4f\n", balances[XLP], balances[ONELP])
	}

	real_egld_in_lp, real_legld_in_lp := float64(0), float64(0)
	onePair, _ := onedex.ViewPair(onedexPairID)
	tmp := big2float(onePair.Second_token_reserve) * balances[ONELP] / big2float(onePair.Lp_token_supply)
	fmt.Printf("egld in onedex %.4f\n", tmp)
	real_egld_in_lp += tmp
	tmp = big2float(onePair.First_token_reserve) * balances[ONELP] / big2float(onePair.Lp_token_supply)
	fmt.Printf("legld in onedex %.4f\n", tmp)
	real_legld_in_lp += tmp

	x1, x2, xlp, _ := xexchange.GetReservesAndTotalSupply()
	tmp = big2float(x2) * balances[XLP] / big2float(xlp)
	fmt.Printf("egld in xexchange %.4f\n", tmp)
	real_egld_in_lp += tmp
	tmp = big2float(x1) * balances[XLP] / big2float(xlp)
	fmt.Printf("legld in xexchange %.4f\n", tmp)
	real_legld_in_lp += tmp

	fmt.Printf("real egld in lp %.4f\n", real_egld_in_lp)
	fmt.Printf("real legld in lp %.4f\n", real_legld_in_lp)

	egld_in_lp, err := readBigIntKey("egld_in_lp")
	if err == nil {
		fmt.Printf("egld in lp %.4f\n", big2float(egld_in_lp))
	}

	legld_in_lp, err := readBigIntKey("legld_in_lp")
	if err == nil {
		fmt.Printf("legld in lp %.4f\n", big2float(legld_in_lp))
	}

	excess_egld_in_lp, err := readBigIntKey("excess_lp_egld")
	if err == nil {
		fmt.Printf("excess egld in lp %.4f\n", big2float(excess_egld_in_lp))
	}

	excess_legld_in_lp, err := readBigIntKey("excess_lp_legld")
	if err == nil {
		fmt.Printf("excess legld in lp %.4f\n", big2float(excess_legld_in_lp))
	}

	egld_profit := real_egld_in_lp - big2float(egld_in_lp)
	legld_profit := real_legld_in_lp - big2float(legld_in_lp)

	price, _ := salsa.GetTokenPrice()

	if legld_profit < 0 {
		// delegate
		egld_profit += legld_profit * big2float(price)
		legld_profit = 0
	}

	if egld_profit < 0 {
		// undelegate
		legld_profit += egld_profit / big2float(price) * 0.98
		egld_profit = 0
	}

	fmt.Printf("profit %.4f\n", egld_profit+legld_profit)

	return nil
}
