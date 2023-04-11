package main

import (
	"bytes"
	"context"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io/ioutil"
	"math/big"
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

const (
	scAddress    = "erd1qqqqqqqqqqqqqpgqwn629zgxxhkvuyu7kafky7swesxlgd00vcqsyee0cw"
	proxyAddress = "https://testnet-gateway.multiversx.com"
	walletFile   = "/home/mihai/walletKey.pem"
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
	undelegates          map[string][]float64 = make(map[string][]float64)
	reserves             map[string]float64   = make(map[string]float64)

	suite  = ed25519.NewEd25519()
	keyGen = signing.NewKeyGenerator(suite)
)

// gas limits
// delegate              30000000
// unDelegate            30000000
// withdraw              40000000
// addReserve             5000000
// removeReserve          5000000
// unDelegateNow         30000000
// compound              30000000
// updateTotalEgldStaked 30000000

func scenario1() error {
	// return nil

	nonce, err := getNonce()
	if err != nil {
		return err
	}

	return compound(30000000, int64(nonce))

	// return configSC("TESTTEST", "TEST", 18, "erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqphllllsndz99p", 5, int64(nonce))

	// for i := 0; i < 100; i++ {
	// 	if err = delegate(10, 30000000, int64(nonce)); err != nil {
	// 		return err
	// 	}
	// 	nonce++
	// }

	// time.Sleep(time.Second * 30)

	// for i := 0; i < 50; i++ {
	// 	if err = unDelegate(10, 30000000, int64(nonce)); err != nil {
	// 		return err
	// 	}
	// 	nonce++
	// }

	/////////////////////////////////////////////////////////////////////////////////////////////

	// for i := 0; i < 30; i++ {
	// 	if err = addReserve(10, 30000000, int64(nonce)); err != nil {
	// 		return err
	// 	}
	// 	nonce++
	// }

	// time.Sleep(time.Second * 30)

	// for i := 0; i < 15; i++ {
	// 	if err = removeReserve(10, 30000000, int64(nonce)); err != nil {
	// 		return err
	// 	}
	// 	nonce++
	// }

	// return unDelegateNow(100, 30000000, int64(nonce))

	return nil
}

func main() {
	err := initialize()
	if err != nil {
		panic(err)
	}

	err = initSC()
	if err != nil {
		panic(err)
	}

	err = readSC()
	if err != nil {
		panic(err)
	}

	fmt.Println("SC address: " + scAddress)
	fmt.Printf("SC active: %v\n", state)
	fmt.Println("Provider address: " + providerAddress)
	fmt.Printf("Undelegate now fee: %.2f%%\n", fee)
	fmt.Println("Token: " + token)
	fmt.Printf("Token supply: %.2f\n", liquidSupply)
	fmt.Printf("Token price: %.2f eGLD\n", liquidSupply/egldStaked)
	fmt.Printf("Total eGLD staked: %.2f\n", egldStaked)
	fmt.Printf("%v undelegations\n", len(undelegates))
	for address, amounts := range undelegates {
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
	fmt.Printf("Total eGLD reserves: %.2f\n", egldReserve)
	fmt.Printf("Available eGLD reserves: %.2f\n", availableEgldReserve)
	fmt.Printf("%v reserves\n", len(reserves))
	for address, amount := range reserves {
		fmt.Printf("%s - %.2f\n", address, amount)
	}

	err = scenario1()
	if err != nil {
		panic(err)
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

func getAccountKeys(address string, prefix string) (map[string][]byte, error) {
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

	result := make(map[string][]byte)
	for key, value := range response.Data.Pairs {
		bv, err := hex.DecodeString(value)
		if err != nil {
			return nil, err
		}

		if strings.HasPrefix(key, prefix) {
			result[key] = bv
		}
	}

	return result, nil
}

func initSC() error {
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
	keys, err := getAccountKeys(scAddress, key)
	if err != nil || len(keys) != 1 {
		return err
	}

	token = string(keys[key])

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

	prefix := []byte("user_undelegations")
	searchKey := hex.EncodeToString(prefix)
	keys, err := getAccountKeys(scAddress, searchKey)
	if err != nil {
		return err
	}

	conv, _ := pubkeyConverter.NewBech32PubkeyConverter(32, logger.GetOrCreate("salsa"))
	undelegates = make(map[string][]float64)
	for key, value := range keys {
		idx := 0
		for {
			key = strings.TrimPrefix(key, hex.EncodeToString(prefix))
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
			if undelegates[address] == nil {
				undelegates[address] = make([]float64, 0)
			}
			undelegates[address] = append(undelegates[address], amount)

			if idx >= len(value) {
				break
			}
		}
	}

	searchKey = hex.EncodeToString([]byte("user_reserves"))
	keys, err = getAccountKeys(scAddress, searchKey)
	if err != nil || len(keys) != 1 {
		return err
	}

	reserves = make(map[string]float64)
	value := keys[searchKey]
	idx := 0
	for {
		var pubKey []byte
		var ok bool
		var iAmount *big.Int
		pubKey, idx, ok = parsePubkey(value, idx)
		allOk := ok
		iAmount, idx, ok = parseBigInt(value, idx)
		allOk = allOk && ok
		if !allOk {
			return errors.New("not all ok")
		}

		address := conv.Encode(pubKey)
		amount := big2float(iAmount, 18)
		reserves[address] = amount

		if idx >= len(value) {
			break
		}
	}

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

func sendTx(value float64, gasLimit uint64, dataField string, nonce int64) (string, error) {
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

	txArgs.RcvAddr = scAddress
	txArgs.Value = float2big(value, 18).String()
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
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")

	client := http.DefaultClient
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	defer resp.Body.Close()

	resBody, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != 200 {
		return resBody, fmt.Errorf("http error %v %v, endpoint %s", resp.StatusCode, resp.Status, address)
	}

	return resBody, nil
}

func delegate(amount float64, gas uint64, nonce int64) error {
	hash, err := sendTx(amount, gas, "delegate", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("delegate %.2f %s\n", amount, hash)

	return nil
}

func unDelegate(amount float64, gas uint64, nonce int64) error {
	dataField := fmt.Sprintf("ESDTTransfer@%s@%s@%s",
		hex.EncodeToString([]byte(token)),
		float2hex(amount, 18),
		hex.EncodeToString([]byte("unDelegate")))
	hash, err := sendTx(0, gas, dataField, nonce)
	if err != nil {
		return err
	}

	fmt.Printf("unDelegate %.2f %s\n", amount, hash)

	return nil
}

func compound(gas uint64, nonce int64) error {
	hash, err := sendTx(0, gas, "compound", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("compound %s\n", hash)

	return nil
}

func withdraw(gas uint64, nonce int64) error {
	hash, err := sendTx(0, gas, "withdraw", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("withdraw %s\n", hash)

	return nil
}

func withdrawAll(gas uint64, nonce int64) error {
	hash, err := sendTx(0, gas, "withdrawAll", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("withdrawAll %s\n", hash)

	return nil
}

func addReserve(amount float64, gas uint64, nonce int64) error {
	hash, err := sendTx(amount, gas, "addReserve", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("addReserve %.2f %s\n", amount, hash)

	return nil
}

func unDelegateNow(amount float64, gas uint64, nonce int64) error {
	dataField := fmt.Sprintf("ESDTTransfer@%s@%s@%s",
		hex.EncodeToString([]byte(token)),
		float2hex(amount, 18),
		hex.EncodeToString([]byte("unDelegateNow")))
	hash, err := sendTx(0, gas, dataField, nonce)
	if err != nil {
		return err
	}

	fmt.Printf("unDelegateNow %.2f %s\n", amount, hash)

	return nil
}

func removeReserve(amount float64, gas uint64, nonce int64) error {
	dataField := fmt.Sprintf("removeReserve@%s", float2hex(amount, 18))
	hash, err := sendTx(0, gas, dataField, nonce)
	if err != nil {
		return err
	}

	fmt.Printf("removeReserve %.2f %s\n", amount, hash)

	return nil
}

func updateTotalEgldStaked(gas uint64, nonce int64) error {
	hash, err := sendTx(0, gas, "updateTotalEgldStaked", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("updateTotalEgldStaked %s\n", hash)

	return nil
}

func configSC(tokenName string, ticker string, decimals int64, provider string, undelegateNowFee float64, nonce int64) error {
	if err := registerLiquidToken(tokenName, ticker, decimals, nonce); err != nil {
		return err
	}

	if nonce != -1 {
		nonce++
	}
	if err := setProviderAddress(provider, nonce); err != nil {
		return err
	}

	if nonce != -1 {
		nonce++
	}
	if err := setUndelegateNowFee(undelegateNowFee, nonce); err != nil {
		return err
	}

	if nonce != -1 {
		nonce++
	}
	time.Sleep(time.Second * 30)

	return setStateActive(nonce)
}

func registerLiquidToken(tokenName string, ticker string, decimals int64, nonce int64) error {
	dataField := fmt.Sprintf("registerLiquidToken@%s@%s@%s",
		hex.EncodeToString([]byte(tokenName)), hex.EncodeToString([]byte(ticker)), hex.EncodeToString(big.NewInt(decimals).Bytes()))
	hash, err := sendTx(0.05, 100000000, dataField, nonce)
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

	hash, err := sendTx(0, 5000000, dataField, nonce)
	if err != nil {
		return err
	}

	fmt.Printf("setProviderAddress %s\n", hash)

	return nil
}

func setUndelegateNowFee(undelegateNowFee float64, nonce int64) error {
	iFee := int64(undelegateNowFee * 100)
	dataField := fmt.Sprintf("setUndelegateNowFee@%s", hex.EncodeToString(big.NewInt(iFee).Bytes()))
	hash, err := sendTx(0, 5000000, dataField, nonce)
	if err != nil {
		return err
	}

	fmt.Printf("setUndelegateNowFee %s\n", hash)

	return nil
}

func setStateActive(nonce int64) error {
	hash, err := sendTx(0, 5000000, "setStateActive", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("setStateActive %s\n", hash)

	return nil
}

func setStateInactive(nonce int64) error {
	hash, err := sendTx(0, 5000000, "setStateInactive", nonce)
	if err != nil {
		return err
	}

	fmt.Printf("setStateInactive %s\n", hash)

	return nil
}
