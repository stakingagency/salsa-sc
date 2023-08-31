package helpers

import (
	"github.com/stakingagency/sa-mx-sdk-go/accounts"
	"github.com/stakingagency/salsa-sc/go-tests/oneDex"
	"github.com/stakingagency/salsa-sc/go-tests/salsaContract"
	"github.com/stakingagency/salsa-sc/go-tests/xExchange"
)

var (
	salsa     *salsaContract.SalsaContract
	onedex    *oneDex.OneDex
	xexchange *xExchange.Pair
	scAccount *accounts.Account
	pk        []byte
	tokenID   string
)
