package main

import (
	"fmt"

	"github.com/stakingagency/salsa-sc/go-tests/helpers"
)

func main() {
	err := helpers.InitSC()
	if err != nil {
		fmt.Println(err)
		return
	}

	err = helpers.StartArbitrage()
	if err != nil {
		fmt.Println(err)
		return
	}

	helpers.GetLpInfo()

	err = helpers.StartTesting()
	if err != nil {
		fmt.Println(err)
		return
	}
}
