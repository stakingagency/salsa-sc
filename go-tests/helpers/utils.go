package helpers

import "math/big"

func str2big(s string) *big.Int {
	res, _ := big.NewInt(0).SetString(s, 10)

	return res
}

func big2float(b *big.Int) float64 {
	f := big.NewFloat(0).SetInt(b)
	for i := 0; i < 18; i++ {
		f.Quo(f, big.NewFloat(10))
	}
	res, _ := f.Float64()

	return res
}
