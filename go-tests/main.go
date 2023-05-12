package main

import (
	"bytes"
	"context"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io/ioutil"
	"math"
	"math/big"
	"math/rand"
	"net/http"
	"strings"
	"time"

	"github.com/multiversx/mx-chain-core-go/core/pubkeyConverter"
	"github.com/multiversx/mx-chain-crypto-go/signing"
	"github.com/multiversx/mx-chain-crypto-go/signing/ed25519"
	logger "github.com/multiversx/mx-chain-logger-go"
	"github.com/multiversx/mx-sdk-go/blockchain"
	"github.com/multiversx/mx-sdk-go/blockchain/cryptoProvider"
	"github.com/multiversx/mx-sdk-go/builders"
	"github.com/multiversx/mx-sdk-go/core"
	"github.com/multiversx/mx-sdk-go/data"
	"github.com/multiversx/mx-sdk-go/interactors"
)

type accountKeys struct {
	Data struct {
		BlockInfo struct {
			Hash     string `json:"hash"`
			Nonce    uint64 `json:"nonce"`
			RootHash string `json:"rootHash"`
		} `json:"blockInfo"`
		Pairs map[string]string `json:"pairs"`
	} `json:"data"`
	Error string `json:"error"`
	Code  string `json:"code"`
}

type accountKey struct {
	Data struct {
		Value string `json:"value"`
	} `json:"data"`
	Error string `json:"error"`
	Code  string `json:"code"`
}

const (
	scAddress = "erd1qqqqqqqqqqqqqpgqefg3dmn0xzf9f6aq2a402knptrtf86qlvcqs3vvzzd"
	// proxyAddress = "http://localhost:8079"
	proxyAddress = "http://193.70.44.72:8079"
	// proxyAddress = "https://devnet-gateway.multiversx.com"
	walletFile = "/home/mihai/walletKey.pem"
	mnemonic   = "asdfghjkl"

	testN = 1000
)

var (
	proxy         blockchain.Proxy
	netCfg        *data.NetworkConfig
	walletAddress string
	privateKey    []byte

	providerAddress string
	token           string
	fee             float64

	state                bool
	egldStaked           float64
	egldReserve          float64
	availableEgldReserve float64
	liquidSupply         float64
	user_undelegates     map[string][]float64 = make(map[string][]float64)
	reserves_undelegates []float64            = make([]float64, 0)
	reserves             map[string]float64   = make(map[string]float64)

	suite  = ed25519.NewEd25519()
	keyGen = signing.NewKeyGenerator(suite)
)

func checkTestResults(idx int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	balance := float64(0)
	if b, err := tAccount.GetBalance(18); err == nil {
		balance += b
	} else {
		return err
	}

	if b, err := getUserTokenBalance(tWalletAddress); err == nil {
		if b != nil {
			balance += big2float(b, 18)
		}
	} else {
		return err
	}

	if b, err := getUserUndelegations(tWalletAddress); err == nil {
		balance += b
	} else {
		return err
	}

	if b, err := getUserReserves(tWalletAddress); err == nil {
		if b != nil {
			balance += big2float(b, 18)
		}
	} else {
		return err
	}

	fmt.Printf("%v %s %.2f\n", idx, tWalletAddress, balance)
	if balance < 99 {
		return errors.New("low balance for address " + tWalletAddress)
	}

	return nil
}

func withdrawTester(idx int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	fmt.Printf("%v ", idx)

	return withdraw(20000000, int64(tAccount.Nonce), tPrivateKey, tWalletAddress)
}

func undelegateAllTester(idx int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	balance, err := getUserTokenBalance(tWalletAddress)
	if err != nil {
		return err
	}

	if balance == nil || balance.Cmp(big.NewInt(0)) == 0 {
		return nil
	}

	fmt.Printf("%v ", idx)

	return unDelegate(balance, 10000000, int64(tAccount.Nonce), tPrivateKey, tWalletAddress)
}

func removeReserveTester(idx int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	balance, err := getUserReserves(tWalletAddress)
	if err != nil {
		return err
	}

	if balance == nil || balance.Cmp(big.NewInt(0)) == 0 {
		return nil
	}

	fmt.Printf("%v ", idx)

	return removeReserve(balance, 10000000, int64(tAccount.Nonce), tPrivateKey, tWalletAddress)
}

func addReserveTester(idx int, amount *big.Int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	return addReserve(amount, 10000000, int64(tAccount.Nonce), tPrivateKey, tWalletAddress)
}

func unDelegateNowTester(idx int, amount *big.Int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	return unDelegateNow(amount, 10000000, int64(tAccount.Nonce), tPrivateKey, tWalletAddress)
}

func delegateTester(idx int, amount *big.Int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	return delegate(amount, 30000000, int64(tAccount.Nonce), tPrivateKey, tWalletAddress)
}

func test(idx int) error {
	w := interactors.NewWallet()
	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(idx))
	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	tWalletAddress := tAddress.AddressAsBech32String()
	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	if err != nil {
		return err
	}

	tNonce := tAccount.Nonce

	for i := 0; i < 10; i++ {
		op := rand.Intn(10)
		switch op {
		case 0:
			if err = delegate(big.NewInt(1000000000000000000), 30000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 1:
			if err = addReserve(big.NewInt(1000000000000000000), 10000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 2:
			b, err := getUserTokenBalance(tWalletAddress)
			if err != nil {
				return err
			}
			if b == nil {
				b = big.NewInt(0)
			}
			if b.Cmp(big.NewInt(1000000000000000000)) < 0 {
				i--
				continue
			}
			if err = unDelegate(big.NewInt(1000000000000000000), 10000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 3:
			b, err := getUserReserves(tWalletAddress)
			if err != nil {
				return err
			}
			if b == nil {
				b = big.NewInt(0)
			}
			if b.Cmp(big.NewInt(1000000000000000000)) < 0 {
				i--
				continue
			}
			if err = removeReserve(big.NewInt(1000000000000000000), 10000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 4:
			b, err := getUserTokenBalance(tWalletAddress)
			if err != nil {
				return err
			}
			if b == nil {
				b = big.NewInt(0)
			}
			if b.Cmp(big.NewInt(1000000000000000000)) < 0 {
				i--
				continue
			}
			if err = unDelegateNow(big.NewInt(1000000000000000000), 10000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 5:
			if err = withdraw(50000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 6:
			if err = withdrawAll(30000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 7:
			if err = compound(30000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 8:
			if err = unDelegateAll(30000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		case 9:
			if err = computeWithdrawn(10000000, int64(tNonce), tPrivateKey, tWalletAddress); err != nil {
				return err
			}
		}
		tNonce++
	}

	return nil
}

func scenario1() error {
	// removeReserve(big.NewInt(1000000000000000000), 200000000, -1, privateKey, walletAddress)

	// nonce, err := getNonce()
	// if err != nil {
	// 	return err
	// }

	// removeReserve(big.NewInt(1000000000000000000), 10000000, -1, privateKey, walletAddress)

	// addReserve(big.NewInt(9000000000000000000), 10000000, -1, privateKey, walletAddress)
	// addReserveTester(1, big.NewInt(1000000000000000000))
	// addReserveTester(2, big.NewInt(2000000000000000000))
	// addReserveTester(3, big.NewInt(3000000000000000000))
	// removeReserveTester(1)
	// delegate(big.NewInt(9000000000000000000), 30000000, -1, privateKey, walletAddress)
	// unDelegateNow(big.NewInt(1000000000000000000), 50000000, -1, privateKey, walletAddress)
	// withdraw(10000000, -1, privateKey, walletAddress)

	// return setStateActive(-1)

	// for i := 0; i < 10; i++ {
	// 	compound(50000000, int64(nonce))
	// 	nonce++
	// 	withdrawAll(200000000, int64(nonce))
	// 	nonce++
	// 	undelegateReserves(200000000, int64(nonce))
	// 	nonce++
	// }

	// go func() {
	// 	for {
	// 		undelegateReserves(100000000, -1)
	// 		time.Sleep(time.Minute * 10)
	// 	}
	// }()
	// time.Sleep(time.Second * 30)
	// for {
	// 	compound(100000000, -1)
	// 	time.Sleep(time.Second * 30)

	// 	updateTotalEgldStaked(100000000, -1)
	// 	time.Sleep(time.Minute)

	// 	withdrawAll(200000000, -1)
	// 	time.Sleep(time.Hour*2 - time.Second*84)
	// }

	// return configSC("TESTTEST", "TEST", 18, "erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqx0llllsdx93z0", 2, int64(nonce))

	return nil
}

func main() {
	err := initialize()
	if err != nil {
		panic(err)
	}

	err = initSC()
	if err != nil {
		// panic(err)
	}

	err = readSC()
	if err != nil {
		// panic(err)
	}

	fmt.Println("SC address: " + scAddress)
	fmt.Printf("SC active: %v\n", state)
	fmt.Println("Provider address: " + providerAddress)
	fmt.Printf("Undelegate now fee: %.2f%%\n", fee)
	fmt.Println("Token: " + token)
	fmt.Printf("Token supply: %.2f\n", liquidSupply)
	fmt.Printf("Token price: %.6f eGLD\n", egldStaked/liquidSupply)
	fmt.Printf("Total eGLD staked: %.2f\n", egldStaked)
	fmt.Printf("%v user undelegations\n", len(user_undelegates))
	for address, amounts := range user_undelegates {
		fmt.Printf("%s -", address)
		total := float64(0)
		count := 0
		for _, amount := range amounts {
			fmt.Printf(" %.2f", amount)
			total += amount
			count++
		}
		fmt.Printf("\n    total = %.2f in %v txs\n", total, count)
	}
	fmt.Printf("%v reserve undelegations\n", len(reserves_undelegates))
	totalReserveUndelegations := float64(0)
	for _, reserveUndelegation := range reserves_undelegates {
		fmt.Printf("%.2f ", reserveUndelegation)
		totalReserveUndelegations += reserveUndelegation
	}
	fmt.Printf("\nTotal reserve undelegations: %.18f\n", totalReserveUndelegations)
	fmt.Printf("%v reserves\n", len(reserves))
	computedReserves := float64(0)
	for address, amount := range reserves {
		fmt.Printf("%s - %.2f\n", address, amount)
		computedReserves += amount
	}
	fmt.Printf("Total eGLD reserves: %.18f\n", egldReserve)
	fmt.Printf("Computed total reserves: %.18f\n", computedReserves)
	fmt.Printf("Available eGLD reserves: %.2f\n", availableEgldReserve)

	// safety checks
	if math.Abs(egldReserve-computedReserves) > 0.000001 {
		fmt.Println("⚠️ egld reserve mismatch (1)")
	}
	if math.Abs(totalReserveUndelegations+availableEgldReserve-egldReserve) > 0.000001 {
		fmt.Println("⚠️ egld reserve mismatch (2)")
	}

	// FUND WALLETS
	// nonce, err := getNonce()
	// if err != nil {
	// 	panic(err)
	// }
	// for i := 0; i < testN; i++ {
	// 	w := interactors.NewWallet()
	// 	tPrivateKey := w.GetPrivateKeyFromMnemonic(mnemonic, 0, uint32(i))
	// 	tAddress, _ := w.GetAddressFromPrivateKey(tPrivateKey)
	// 	tWalletAddress := tAddress.AddressAsBech32String()
	// 	tAccount, err := proxy.GetAccount(context.Background(), tAddress)
	// 	if err != nil {
	// 		continue
	// 	}
	// 	balance, ok := big.NewInt(0).SetString(tAccount.Balance, 10)
	// 	if !ok {
	// 		continue
	// 	}
	// 	b100, _ := big.NewInt(0).SetString("100000000000000000000", 10)
	// 	if b100.Cmp(balance) > 0 {
	// 		b100.Sub(b100, balance)
	// 		sendTx(b100, 50000, "", int64(nonce), privateKey, walletAddress, tWalletAddress)
	// 		nonce++
	// 	} else if b100.Cmp(balance) < 0 {
	// 		balance.Sub(balance, b100)
	// 		sendTx(balance, 50000, "", -1, tPrivateKey, tWalletAddress, walletAddress)
	// 	}
	// }
	// time.Sleep(time.Second * 30)

	// err = scenario1()
	// if err != nil {
	// 	fmt.Println(err)
	// }
	// os.Exit(1)

	// CHECK TEST RESULTS
	// for i := 0; i < testN; i++ {
	// 	if err := checkTestResults(i); err != nil {
	// 		fmt.Println(err)
	// 	}
	// }

	// WITHDRAW EACH
	// for i := 0; i < testN; i++ {
	// 	if err := withdrawTester(i); err != nil {
	// 		// panic(err)
	// 	}
	// }

	// UNDELEGATE EACH
	// for i := 0; i < testN; i++ {
	// 	if err := undelegateAllTester(i); err != nil {
	// 		// panic(err)
	// 	}
	// }

	// REMOVE RESERVE EACH
	// for i := 0; i < testN; i++ {
	// 	if err := removeReserveTester(i); err != nil {
	// 		// panic(err)
	// 	}
	// }

	// STRESS TEST
	for {
		for i := 0; i < testN; i++ {
			go func(i int) {
				err = test(i)
				if err != nil {
					fmt.Println(err)
				}
			}(i)
		}
		time.Sleep(time.Minute * 30)
	}
}

func queryVM(scAddress, funcName string, args []string) ([]byte, error) {
	request := &data.VmValueRequest{
		Address:  scAddress,
		FuncName: funcName,
		Args:     args,
	}
	res, err := proxy.ExecuteVMQuery(context.Background(), request)
	if err != nil {
		return nil, err
	}

	if len(res.Data.ReturnData) == 0 {
		return []byte{}, nil
	}

	return res.Data.ReturnData[0], nil
}

var allKeys = make(map[string][]byte)

func getAccountKeys(address string, prefix string) (map[string][]byte, error) {
	result := make(map[string][]byte)
	if prefix == "" {
		endpoint := fmt.Sprintf("%s/address/%s/keys", proxyAddress, address)
		bytes, err := getHTTP(endpoint, "")
		if err != nil {
			return nil, err
		}

		response := &accountKeys{}
		err = json.Unmarshal(bytes, response)
		if err != nil {
			return nil, err
		}

		if response.Error != "" {
			return nil, errors.New(response.Error)
		}

		for key, value := range response.Data.Pairs {
			bv, err := hex.DecodeString(value)
			if err != nil {
				return nil, err
			}

			if strings.HasPrefix(key, prefix) {
				result[key] = bv
			}
		}
	} else {
		for k, v := range allKeys {
			if strings.HasPrefix(k, prefix) {
				result[k] = v
			}
		}
	}

	return result, nil
}

func getAccountKey(address string, key string) ([]byte, error) {
	endpoint := fmt.Sprintf("%s/address/%s/key/%s", proxyAddress, address, key)
	bytes, err := getHTTP(endpoint, "")
	if err != nil {
		return nil, err
	}

	response := &accountKey{}
	err = json.Unmarshal(bytes, response)
	if err != nil {
		return nil, err
	}

	if response.Error != "" {
		return nil, errors.New(response.Error)
	}

	return hex.DecodeString(response.Data.Value)
}

func initSC() error {
	var err error
	allKeys, err = getAccountKeys(scAddress, "")
	if err != nil {
		return err
	}

	bFee, err := queryVM(scAddress, "getUndelegateNowFee", []string{})
	if err != nil {
		return err
	}

	iFee := big.NewInt(0).SetBytes(bFee)
	fee = float64(iFee.Uint64()) / 100
	iAddress, err := queryVM(scAddress, "getProviderAddress", []string{})
	if err != nil {
		return err
	}

	conv, _ := pubkeyConverter.NewBech32PubkeyConverter(32, logger.GetOrCreate("salsa"))
	providerAddress = conv.Encode(iAddress)

	key := hex.EncodeToString([]byte("liquid_token_id"))
	keys, err := getAccountKey(scAddress, key)
	if err != nil {
		return err
	}

	token = string(keys)

	return nil
}

func readSC() error {
	bState, err := queryVM(scAddress, "getState", []string{})
	if err != nil {
		return err
	}

	iState := big.NewInt(0).SetBytes(bState)
	state = iState.Uint64() == 1

	bEgldStaked, err := queryVM(scAddress, "getTotalEgldStaked", []string{})
	if err != nil {
		return err
	}

	iEgldStaked := big.NewInt(0).SetBytes(bEgldStaked)
	egldStaked = big2float(iEgldStaked, 18)

	bEgldReserve, err := queryVM(scAddress, "getEgldReserve", []string{})
	if err != nil {
		return err
	}

	iEgldReserve := big.NewInt(0).SetBytes(bEgldReserve)
	egldReserve = big2float(iEgldReserve, 18)

	bAvailableEgldReserve, err := queryVM(scAddress, "getAvailableEgldReserve", []string{})
	if err != nil {
		return err
	}

	iAvailableEgldReserve := big.NewInt(0).SetBytes(bAvailableEgldReserve)
	availableEgldReserve = big2float(iAvailableEgldReserve, 18)

	bLiquidTokenSupply, err := queryVM(scAddress, "getLiquidTokenSupply", []string{})
	if err != nil {
		return err
	}

	iLiquidTokenSupply := big.NewInt(0).SetBytes(bLiquidTokenSupply)
	liquidSupply = big2float(iLiquidTokenSupply, 18)

	searchKey := hex.EncodeToString([]byte("user_undelegations"))
	keys, err := getAccountKeys(scAddress, searchKey)
	if err != nil {
		return err
	}

	conv, _ := pubkeyConverter.NewBech32PubkeyConverter(32, logger.GetOrCreate("salsa"))
	user_undelegates = make(map[string][]float64)
	for key, value := range keys {
		idx := 0
		for {
			key = strings.TrimPrefix(key, searchKey)
			var iAmount *big.Int
			var ok bool
			iAmount, idx, ok = parseBigInt(value, idx)
			allOk := ok
			_, idx, ok = parseUint64(value, idx)
			allOk = allOk && ok
			if !allOk {
				return errors.New("not all ok")
			}

			pubKey, _ := hex.DecodeString(key)
			address := conv.Encode(pubKey)
			amount := big2float(iAmount, 18)
			if user_undelegates[address] == nil {
				user_undelegates[address] = make([]float64, 0)
			}
			user_undelegates[address] = append(user_undelegates[address], amount)

			if idx >= len(value) {
				break
			}
		}
	}

	searchKey = hex.EncodeToString([]byte("reservers_addresses"))
	keys, err = getAccountKeys(scAddress, searchKey)
	if err != nil {
		return err
	}

	reserves = make(map[string]float64)
	for key, value := range keys {
		key = strings.TrimPrefix(key, searchKey)
		pubKey, _ := hex.DecodeString(key)
		index := big.NewInt(0).SetBytes(value)
		searchKey2 := hex.EncodeToString([]byte("users_reserves.item")) + fmt.Sprintf("%.8x", index)
		keys2, err := getAccountKey(scAddress, searchKey2)
		if err != nil {
			return err
		}
		iAmount := big.NewInt(0).SetBytes(keys2)
		address := conv.Encode(pubKey)
		amount := big2float(iAmount, 18)
		reserves[address] = amount
	}

	searchKey = hex.EncodeToString([]byte("reserve_undelegations"))
	keys, err = getAccountKeys(scAddress, searchKey)
	if err != nil || len(keys) != 1 {
		return err
	}

	reserves_undelegates = make([]float64, 0)
	value := keys[searchKey]
	idx := 0
	for {
		var ok bool
		var iAmount *big.Int
		iAmount, idx, ok = parseBigInt(value, idx)
		allOk := ok
		_, idx, ok = parseUint64(value, idx)
		allOk = allOk && ok
		if !allOk {
			return errors.New("not all ok")
		}

		amount := big2float(iAmount, 18)
		reserves_undelegates = append(reserves_undelegates, amount)

		if idx >= len(value) {
			break
		}
	}
	allKeys = make(map[string][]byte)

	return nil
}

func big2float(value *big.Int, decimals int) float64 {
	f := big.NewFloat(0).SetInt(value)
	for i := 0; i < decimals; i++ {
		f.Quo(f, big.NewFloat(10))
	}
	res, _ := f.Float64()

	return res
}

func float2big(value float64, decimals int) *big.Int {
	f := big.NewFloat(value)
	for i := 0; i < decimals; i++ {
		f.Mul(f, big.NewFloat(10))
	}
	res, _ := f.Int(nil)

	return res
}

func initialize() error {
	var err error

	args := blockchain.ArgsProxy{
		ProxyURL:            proxyAddress,
		Client:              nil,
		SameScState:         false,
		ShouldBeSynced:      false,
		FinalityCheck:       false,
		CacheExpirationTime: time.Minute,
		EntityType:          core.Proxy,
	}
	proxy, err = blockchain.NewProxy(args)
	if err != nil {
		return err
	}

	netCfg, err = proxy.GetNetworkConfig(context.Background())
	if err != nil {
		return err
	}

	w := interactors.NewWallet()
	privateKey, err = w.LoadPrivateKeyFromPemFile(walletFile)
	if err != nil {
		return err
	}

	address, _ := w.GetAddressFromPrivateKey(privateKey)
	walletAddress = address.AddressAsBech32String()

	return nil
}

func float2hex(value float64, decimals int) string {
	bigValue := big.NewFloat(value)
	for i := 0; i < decimals; i++ {
		bigValue.Mul(bigValue, big.NewFloat(10))
	}
	iValue, _ := bigValue.Int(nil)

	return hex.EncodeToString(iValue.Bytes())
}

func str2hex(s string) string {
	return hex.EncodeToString([]byte(s))
}

func getNonce() (uint64, error) {
	address, _ := data.NewAddressFromBech32String(walletAddress)
	account, err := proxy.GetAccount(context.Background(), address)
	if err != nil {
		return 0, err
	}

	return account.Nonce, nil
}

func sendTx(value *big.Int, gasLimit uint64, dataField string, nonce int64, privateKey []byte, walletAddress string, receiver string) (string, error) {
	args := blockchain.ArgsProxy{
		ProxyURL:            proxyAddress,
		Client:              nil,
		SameScState:         false,
		ShouldBeSynced:      false,
		FinalityCheck:       false,
		CacheExpirationTime: time.Minute,
		EntityType:          core.Proxy,
	}
	proxy, err := blockchain.NewProxy(args)
	if err != nil {
		return "", err
	}

	address, _ := data.NewAddressFromBech32String(walletAddress)

	txArgs, err := proxy.GetDefaultTransactionArguments(context.Background(), address, netCfg)
	if err != nil {
		return "", err
	}

	if nonce != -1 {
		txArgs.Nonce = uint64(nonce)
	}

	if receiver == "" {
		txArgs.RcvAddr = scAddress
	} else {
		txArgs.RcvAddr = receiver
	}
	txArgs.Value = value.String()
	txArgs.Data = []byte(dataField)
	txArgs.GasLimit = gasLimit

	holder, _ := cryptoProvider.NewCryptoComponentsHolder(keyGen, privateKey)
	txBuilder, err := builders.NewTxBuilder(cryptoProvider.NewSigner())
	if err != nil {
		return "", err
	}

	ti, err := interactors.NewTransactionInteractor(proxy, txBuilder)
	if err != nil {
		return "", err
	}

	tx, err := ti.ApplySignatureAndGenerateTx(holder, txArgs)
	if err != nil {
		return "", err
	}

	hash, err := ti.SendTransaction(context.Background(), tx)
	if err != nil {
		return "", err
	}

	return hash, nil
}

func getHTTP(address string, body string) ([]byte, error) {
	req, err := http.NewRequest(http.MethodGet, address, bytes.NewBuffer([]byte(body)))
	if err != nil {
		fmt.Println(address)
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")

	client := http.DefaultClient
	resp, err := client.Do(req)
	if err != nil {
		fmt.Println(address)
		return nil, err
	}

	defer resp.Body.Close()

	resBody, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		fmt.Println(address)
		return nil, err
	}

	if resp.StatusCode != 200 {
		fmt.Println(address)
		return resBody, fmt.Errorf("http error %v %v, endpoint %s", resp.StatusCode, resp.Status, address)
	}

	return resBody, nil
}

func delegate(amount *big.Int, gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	hash, err := sendTx(amount, gas, "delegate", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("delegate %.2f %s\n", big2float(amount, 18), hash)

	return nil
}

func unDelegate(amount *big.Int, gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	dataField := fmt.Sprintf("ESDTTransfer@%s@%s@%s",
		hex.EncodeToString([]byte(token)),
		hex.EncodeToString(amount.Bytes()),
		hex.EncodeToString([]byte("unDelegate")))
	hash, err := sendTx(big.NewInt(0), gas, dataField, nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("unDelegate %.2f %s\n", big2float(amount, 18), hash)

	return nil
}

func compound(gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	hash, err := sendTx(big.NewInt(0), gas, "compound", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("compound %s\n", hash)

	return nil
}

func withdraw(gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	hash, err := sendTx(big.NewInt(0), gas, "withdraw", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("withdraw %s\n", hash)

	return nil
}

func withdrawAll(gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	hash, err := sendTx(big.NewInt(0), gas, "withdrawAll", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("withdrawAll %s\n", hash)

	return nil
}

func unDelegateAll(gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	hash, err := sendTx(big.NewInt(0), gas, "unDelegateAll", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("unDelegateAll %s\n", hash)

	return nil
}

func computeWithdrawn(gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	hash, err := sendTx(big.NewInt(0), gas, "computeWithdrawn", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("computeWithdrawn %s\n", hash)

	return nil
}

func addReserve(amount *big.Int, gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	hash, err := sendTx(amount, gas, "addReserve", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("addReserve %.2f %s\n", big2float(amount, 18), hash)

	return nil
}

func unDelegateNow(amount *big.Int, gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	dataField := fmt.Sprintf("ESDTTransfer@%s@%s@%s",
		hex.EncodeToString([]byte(token)),
		hex.EncodeToString(amount.Bytes()),
		hex.EncodeToString([]byte("unDelegateNow")))
	hash, err := sendTx(big.NewInt(0), gas, dataField, nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("unDelegateNow %.2f %s\n", big2float(amount, 18), hash)

	return nil
}

func removeReserve(amount *big.Int, gas uint64, nonce int64, privateKey []byte, walletAddress string) error {
	dataField := fmt.Sprintf("removeReserve@%s", hex.EncodeToString(amount.Bytes()))
	hash, err := sendTx(big.NewInt(0), gas, dataField, nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("removeReserve %.2f %s\n", big2float(amount, 18), hash)

	return nil
}

func configSC(tokenName string, ticker string, decimals int64, provider string, undelegateNowFee float64, nonce int64) error {
	if err := registerLiquidToken(tokenName, ticker, decimals, nonce); err != nil {
		return err
	}

	time.Sleep(time.Second * 40)

	if nonce != -1 {
		nonce++
	}
	if err := setProviderAddress(provider, nonce); err != nil {
		return err
	}

	time.Sleep(time.Second * 6)

	if nonce != -1 {
		nonce++
	}
	if err := setUndelegateNowFee(undelegateNowFee, nonce); err != nil {
		return err
	}

	time.Sleep(time.Second * 6)

	if nonce != -1 {
		nonce++
	}
	time.Sleep(time.Second * 30)

	return setStateActive(nonce)
}

func registerLiquidToken(tokenName string, ticker string, decimals int64, nonce int64) error {
	dataField := fmt.Sprintf("registerLiquidToken@%s@%s@%s",
		hex.EncodeToString([]byte(tokenName)), hex.EncodeToString([]byte(ticker)), hex.EncodeToString(big.NewInt(decimals).Bytes()))
	hash, err := sendTx(big.NewInt(50000000000000000), 100000000, dataField, nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("registerLiquidToken %s\n", hash)

	return nil
}

func setProviderAddress(provider string, nonce int64) error {
	conv, _ := pubkeyConverter.NewBech32PubkeyConverter(32, logger.GetOrCreate("salsa"))
	pubkey, err := conv.Decode(provider)
	if err != nil {
		return err
	}

	dataField := fmt.Sprintf("setProviderAddress@%s", hex.EncodeToString(pubkey))

	hash, err := sendTx(big.NewInt(0), 5000000, dataField, nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("setProviderAddress %s\n", hash)

	return nil
}

func setUndelegateNowFee(undelegateNowFee float64, nonce int64) error {
	iFee := int64(undelegateNowFee * 100)
	dataField := fmt.Sprintf("setUndelegateNowFee@%s", hex.EncodeToString(big.NewInt(iFee).Bytes()))
	hash, err := sendTx(big.NewInt(0), 5000000, dataField, nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("setUndelegateNowFee %s\n", hash)

	return nil
}

func setStateActive(nonce int64) error {
	hash, err := sendTx(big.NewInt(0), 5000000, "setStateActive", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("setStateActive %s\n", hash)

	return nil
}

func setStateInactive(nonce int64) error {
	hash, err := sendTx(big.NewInt(0), 5000000, "setStateInactive", nonce, privateKey, walletAddress, "")
	if err != nil {
		return err
	}

	fmt.Printf("setStateInactive %s\n", hash)

	return nil
}

func getUserUndelegations(walletAddress string) (float64, error) {
	conv, _ := pubkeyConverter.NewBech32PubkeyConverter(32, logger.GetOrCreate("salsa"))
	pubkey, err := conv.Decode(walletAddress)
	if err != nil {
		return 0, err
	}

	key := append([]byte("user_undelegations"), pubkey...)
	searchKey := hex.EncodeToString(key)
	value, err := getAccountKey(scAddress, searchKey)
	if err != nil {
		return 0, err
	}

	undelegates := make([]float64, 0)
	if len(value) > 0 {
		idx := 0
		for {
			var iAmount *big.Int
			var ok bool
			iAmount, idx, ok = parseBigInt(value, idx)
			allOk := ok
			_, idx, ok = parseUint64(value, idx)
			allOk = allOk && ok
			if !allOk {
				return 0, errors.New("not all ok")
			}

			amount := big2float(iAmount, 18)
			undelegates = append(undelegates, amount)

			if idx >= len(value) {
				break
			}
		}
	}

	totalUndelegation := float64(0)
	for _, undelegation := range undelegates {
		totalUndelegation += undelegation
	}

	return totalUndelegation, nil
}

func getUserReserves(walletAddress string) (*big.Int, error) {
	conv, _ := pubkeyConverter.NewBech32PubkeyConverter(32, logger.GetOrCreate("salsa"))
	pubkey, err := conv.Decode(walletAddress)
	if err != nil {
		return nil, err
	}

	searchKey := hex.EncodeToString(append([]byte("users_reserve_points"), pubkey...))
	key, err := getAccountKey(scAddress, searchKey)
	if err != nil {
		return nil, err
	}

	points := big.NewInt(0).SetBytes(key)

	searchKey = hex.EncodeToString([]byte("reserve_points"))
	key, err = getAccountKey(scAddress, searchKey)
	if err != nil {
		return nil, err
	}

	totalPoints := big.NewInt(0).SetBytes(key)

	searchKey = hex.EncodeToString([]byte("egld_reserve"))
	key, err = getAccountKey(scAddress, searchKey)
	if err != nil {
		return nil, err
	}

	totalReserve := big.NewInt(0).SetBytes(key)

	reserve := big.NewInt(0).Set(points)
	reserve.Mul(reserve, totalReserve)
	reserve.Quo(reserve, totalPoints)

	return reserve, nil
}

type tokenBalance struct {
	Data struct {
		TokenData struct {
			Balance string `json:"balance"`
		} `json:"tokenData"`
	} `json:"data"`
}

func getUserTokenBalance(walletAddress string) (*big.Int, error) {
	endpoint := fmt.Sprintf("%s/address/%s/esdt/%s", proxyAddress, walletAddress, token)
	bytes, err := getHTTP(endpoint, "")
	if err != nil {
		return big.NewInt(0), nil
	}

	response := &tokenBalance{}
	err = json.Unmarshal(bytes, response)
	if err != nil {
		return nil, err
	}

	iBalance, ok := big.NewInt(0).SetString(response.Data.TokenData.Balance, 10)
	if !ok {
		return big.NewInt(0), nil
	}

	return iBalance, nil
}
