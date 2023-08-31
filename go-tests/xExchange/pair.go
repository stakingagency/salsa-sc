package xExchange

import (
	"encoding/binary"
	"encoding/hex"
	"errors"
	"math/big"
	"strings"

	"github.com/stakingagency/sa-mx-sdk-go/data"
	"github.com/stakingagency/sa-mx-sdk-go/network"
	"github.com/stakingagency/sa-mx-sdk-go/utils"
)

type Address []byte

type TokenIdentifier string

type SwapEvent struct {
	Caller            Address
	Token_id_in       TokenIdentifier
	Token_amount_in   *big.Int
	Token_id_out      TokenIdentifier
	Token_amount_out  *big.Int
	Fee_amount        *big.Int
	Token_in_reserve  *big.Int
	Token_out_reserve *big.Int
	Block             uint64
	Epoch             uint64
	Timestamp         uint64
}

type SwapNoFeeAndForwardEvent struct {
	Caller           Address
	Token_id_in      TokenIdentifier
	Token_amount_in  *big.Int
	Token_id_out     TokenIdentifier
	Token_amount_out *big.Int
	Destination      Address
	Block            uint64
	Epoch            uint64
	Timestamp        uint64
}

type TokenPair struct {
	First_token  TokenIdentifier
	Second_token TokenIdentifier
}

type ComplexType6 struct {
	Var0 Address
	Var1 TokenIdentifier
}

type ComplexType7 struct {
	Var0 TokenPair
	Var1 Address
}

type AddLiquidityEvent struct {
	Caller                Address
	First_token_id        TokenIdentifier
	First_token_amount    *big.Int
	Second_token_id       TokenIdentifier
	Second_token_amount   *big.Int
	Lp_token_id           TokenIdentifier
	Lp_token_amount       *big.Int
	Lp_supply             *big.Int
	First_token_reserves  *big.Int
	Second_token_reserves *big.Int
	Block                 uint64
	Epoch                 uint64
	Timestamp             uint64
}

type EsdtTokenPayment struct {
	Token_identifier TokenIdentifier
	Token_nonce      uint64
	Amount           *big.Int
}

type RemoveLiquidityEvent struct {
	Caller                Address
	First_token_id        TokenIdentifier
	First_token_amount    *big.Int
	Second_token_id       TokenIdentifier
	Second_token_amount   *big.Int
	Lp_token_id           TokenIdentifier
	Lp_token_amount       *big.Int
	Lp_supply             *big.Int
	First_token_reserves  *big.Int
	Second_token_reserves *big.Int
	Block                 uint64
	Epoch                 uint64
	Timestamp             uint64
}

type State int

const (
	Inactive      State = 0
	Active        State = 1
	PartialActive State = 2
)

type Pair struct {
	netMan          *network.NetworkManager
	contractAddress string
}

func NewPair(contractAddress string, proxyAddress string, indexAddress string) (*Pair, error) {
	netMan, err := network.NewNetworkManager(proxyAddress, indexAddress)
	if err != nil {
		return nil, err
	}

	contract := &Pair{
		netMan:          netMan,
		contractAddress: contractAddress,
	}

	return contract, nil
}

func (contract *Pair) GetNetworkManager() *network.NetworkManager {
	return contract.netMan
}
func (contract *Pair) GetTokensForGivenPosition(liquidity *big.Int) (EsdtTokenPayment, EsdtTokenPayment, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(liquidity.Bytes()))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTokensForGivenPosition", _args)
	if err != nil {
		return EsdtTokenPayment{}, EsdtTokenPayment{}, err
	}

	idx := 0
	ok, allOk := true, true
	_Token_identifier, idx, ok := utils.ParseString(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_Token_nonce, idx, ok := utils.ParseUint64(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_Amount, idx, ok := utils.ParseBigInt(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	if !allOk {
		return EsdtTokenPayment{}, EsdtTokenPayment{}, errors.New("invalid response")
	}

	res0 := EsdtTokenPayment{
		Token_identifier: TokenIdentifier(_Token_identifier),
		Token_nonce:      _Token_nonce,
		Amount:           _Amount,
	}
	idx = 0
	ok, allOk = true, true
	_Token_identifier, idx, ok = utils.ParseString(res.Data.ReturnData[1], idx)
	allOk = allOk && ok
	_Token_nonce, idx, ok = utils.ParseUint64(res.Data.ReturnData[1], idx)
	allOk = allOk && ok
	_Amount, idx, ok = utils.ParseBigInt(res.Data.ReturnData[1], idx)
	allOk = allOk && ok
	if !allOk {
		return EsdtTokenPayment{}, EsdtTokenPayment{}, errors.New("invalid response")
	}

	res1 := EsdtTokenPayment{
		Token_identifier: TokenIdentifier(_Token_identifier),
		Token_nonce:      _Token_nonce,
		Amount:           _Amount,
	}

	return res0, res1, nil
}

func (contract *Pair) GetReservesAndTotalSupply() (*big.Int, *big.Int, *big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getReservesAndTotalSupply", nil)
	if err != nil {
		return nil, nil, nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])
	res1 := big.NewInt(0).SetBytes(res.Data.ReturnData[1])
	res2 := big.NewInt(0).SetBytes(res.Data.ReturnData[2])

	return res0, res1, res2, nil
}

func (contract *Pair) GetAmountOut(token_in TokenIdentifier, amount_in *big.Int) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_in)))
	_args = append(_args, hex.EncodeToString(amount_in.Bytes()))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getAmountOut", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetAmountIn(token_wanted TokenIdentifier, amount_wanted *big.Int) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_wanted)))
	_args = append(_args, hex.EncodeToString(amount_wanted.Bytes()))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getAmountIn", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetEquivalent(token_in TokenIdentifier, amount_in *big.Int) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_in)))
	_args = append(_args, hex.EncodeToString(amount_in.Bytes()))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getEquivalent", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetFeeState() (bool, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getFeeState", nil)
	if err != nil {
		return false, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64() == 1

	return res0, nil
}

func (contract *Pair) GetFeeDestinations() ([]ComplexType6, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getFeeDestinations", nil)
	if err != nil {
		return nil, err
	}

	res0 := make([]ComplexType6, 0)
	for i := 0; i < len(res.Data.ReturnData); i += 2 {
		Var0 := res.Data.ReturnData[i+0]
		Var1 := TokenIdentifier(res.Data.ReturnData[i+1])
		inner := ComplexType6{
			Var0: Var0,
			Var1: Var1,
		}
		res0 = append(res0, inner)
	}

	return res0, nil
}

func (contract *Pair) GetTrustedSwapPairs() ([]ComplexType7, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTrustedSwapPairs", nil)
	if err != nil {
		return nil, err
	}

	res0 := make([]ComplexType7, 0)
	for i := 0; i < len(res.Data.ReturnData); i += 2 {
		idx := 0
		ok, allOk := true, true
		_First_token, idx, ok := utils.ParseString(res.Data.ReturnData[i+0], idx)
		allOk = allOk && ok
		_Second_token, idx, ok := utils.ParseString(res.Data.ReturnData[i+0], idx)
		allOk = allOk && ok
		if !allOk {
			return nil, errors.New("invalid response")
		}

		Var0 := TokenPair{
			First_token:  TokenIdentifier(_First_token),
			Second_token: TokenIdentifier(_Second_token),
		}
		Var1 := res.Data.ReturnData[i+1]
		inner := ComplexType7{
			Var0: Var0,
			Var1: Var1,
		}
		res0 = append(res0, inner)
	}

	return res0, nil
}

func (contract *Pair) GetWhitelistedManagedAddresses() ([]Address, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getWhitelistedManagedAddresses", nil)
	if err != nil {
		return nil, err
	}

	res0 := make([]Address, 0)
	for i := 0; i < len(res.Data.ReturnData); i++ {
		_item := res.Data.ReturnData[i]
		res0 = append(res0, _item)
	}

	return res0, nil
}

func (contract *Pair) GetFeesCollectorAddress() (Address, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getFeesCollectorAddress", nil)
	if err != nil {
		return nil, err
	}

	res0 := res.Data.ReturnData[0]

	return res0, nil
}

func (contract *Pair) GetFeesCollectorCutPercentage() (uint64, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getFeesCollectorCutPercentage", nil)
	if err != nil {
		return 0, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64()

	return res0, nil
}

func (contract *Pair) GetLpTokenIdentifier() (TokenIdentifier, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getLpTokenIdentifier", nil)
	if err != nil {
		return "", err
	}

	res0 := TokenIdentifier(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetTotalFeePercent() (uint64, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTotalFeePercent", nil)
	if err != nil {
		return 0, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64()

	return res0, nil
}

func (contract *Pair) GetSpecialFee() (uint64, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getSpecialFee", nil)
	if err != nil {
		return 0, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64()

	return res0, nil
}

func (contract *Pair) GetRouterManagedAddress() (Address, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getRouterManagedAddress", nil)
	if err != nil {
		return nil, err
	}

	res0 := res.Data.ReturnData[0]

	return res0, nil
}

func (contract *Pair) GetFirstTokenId() (TokenIdentifier, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getFirstTokenId", nil)
	if err != nil {
		return "", err
	}

	res0 := TokenIdentifier(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetSecondTokenId() (TokenIdentifier, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getSecondTokenId", nil)
	if err != nil {
		return "", err
	}

	res0 := TokenIdentifier(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetTotalSupply() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTotalSupply", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetInitialLiquidtyAdder() (Address, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getInitialLiquidtyAdder", nil)
	if err != nil {
		return nil, err
	}

	res0 := res.Data.ReturnData[0]

	return res0, nil
}

func (contract *Pair) GetReserve(token_id TokenIdentifier) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_id)))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getReserve", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *Pair) GetLockingScAddress() (Address, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getLockingScAddress", nil)
	if err != nil {
		return nil, err
	}

	res0 := res.Data.ReturnData[0]

	return res0, nil
}

func (contract *Pair) GetUnlockEpoch() (uint64, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUnlockEpoch", nil)
	if err != nil {
		return 0, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64()

	return res0, nil
}

func (contract *Pair) GetLockingDeadlineEpoch() (uint64, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getLockingDeadlineEpoch", nil)
	if err != nil {
		return 0, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64()

	return res0, nil
}

func (contract *Pair) GetPermissions(address Address) (uint32, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getPermissions", _args)
	if err != nil {
		return 0, err
	}

	res0 := uint32(big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64())

	return res0, nil
}

func (contract *Pair) GetState() (State, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getState", nil)
	if err != nil {
		return 0, err
	}

	res0 := State(big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64())

	return res0, nil
}

func (contract *Pair) AddInitialLiquidity(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := hex.EncodeToString([]byte("addInitialLiquidity"))
	hash, err := contract.netMan.SendEsdtTransaction(_pk, contract.contractAddress, _value, _gasLimit, _token, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) AddLiquidity(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, first_token_amount_min *big.Int, second_token_amount_min *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(first_token_amount_min.Bytes()))
	_args = append(_args, hex.EncodeToString(second_token_amount_min.Bytes()))
	dataField := hex.EncodeToString([]byte("addLiquidity")) + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendEsdtTransaction(_pk, contract.contractAddress, _value, _gasLimit, _token, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) RemoveLiquidity(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, first_token_amount_min *big.Int, second_token_amount_min *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(first_token_amount_min.Bytes()))
	_args = append(_args, hex.EncodeToString(second_token_amount_min.Bytes()))
	dataField := hex.EncodeToString([]byte("removeLiquidity")) + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendEsdtTransaction(_pk, contract.contractAddress, _value, _gasLimit, _token, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) RemoveLiquidityAndBuyBackAndBurnToken(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, token_to_buyback_and_burn TokenIdentifier) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_to_buyback_and_burn)))
	dataField := hex.EncodeToString([]byte("removeLiquidityAndBuyBackAndBurnToken")) + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendEsdtTransaction(_pk, contract.contractAddress, _value, _gasLimit, _token, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SwapNoFeeAndForward(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, token_out TokenIdentifier, destination_address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_out)))
	_args = append(_args, hex.EncodeToString(destination_address))
	dataField := hex.EncodeToString([]byte("swapNoFeeAndForward")) + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendEsdtTransaction(_pk, contract.contractAddress, _value, _gasLimit, _token, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SwapTokensFixedInput(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, token_out TokenIdentifier, amount_out_min *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_out)))
	_args = append(_args, hex.EncodeToString(amount_out_min.Bytes()))
	dataField := hex.EncodeToString([]byte("swapTokensFixedInput")) + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendEsdtTransaction(_pk, contract.contractAddress, _value, _gasLimit, _token, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SwapTokensFixedOutput(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, token_out TokenIdentifier, amount_out *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_out)))
	_args = append(_args, hex.EncodeToString(amount_out.Bytes()))
	dataField := hex.EncodeToString([]byte("swapTokensFixedOutput")) + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendEsdtTransaction(_pk, contract.contractAddress, _value, _gasLimit, _token, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetLpTokenIdentifier(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, token_identifier TokenIdentifier) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_identifier)))
	dataField := "setLpTokenIdentifier" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) Whitelist(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "whitelist" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) RemoveWhitelist(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "removeWhitelist" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) AddTrustedSwapPair(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, pair_address Address, first_token TokenIdentifier, second_token TokenIdentifier) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(pair_address))
	_args = append(_args, hex.EncodeToString([]byte(first_token)))
	_args = append(_args, hex.EncodeToString([]byte(second_token)))
	dataField := "addTrustedSwapPair" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) RemoveTrustedSwapPair(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, first_token TokenIdentifier, second_token TokenIdentifier) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(first_token)))
	_args = append(_args, hex.EncodeToString([]byte(second_token)))
	dataField := "removeTrustedSwapPair" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetupFeesCollector(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, fees_collector_address Address, fees_collector_cut_percentage uint64) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(fees_collector_address))
	bytes164 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes164, fees_collector_cut_percentage)
	_args = append(_args, hex.EncodeToString(bytes164))
	dataField := "setupFeesCollector" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetFeeOn(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, enabled bool, fee_to_address Address, fee_token TokenIdentifier) error {
	_args := make([]string, 0)
	if enabled {
		_args = append(_args, "01")
	} else {
		_args = append(_args, "00")
	}
	_args = append(_args, hex.EncodeToString(fee_to_address))
	_args = append(_args, hex.EncodeToString([]byte(fee_token)))
	dataField := "setFeeOn" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetStateActiveNoSwaps(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setStateActiveNoSwaps"
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetFeePercents(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, total_fee_percent uint64, special_fee_percent uint64) error {
	_args := make([]string, 0)
	bytes064 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes064, total_fee_percent)
	_args = append(_args, hex.EncodeToString(bytes064))
	bytes164 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes164, special_fee_percent)
	_args = append(_args, hex.EncodeToString(bytes164))
	dataField := "setFeePercents" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) UpdateAndGetTokensForGivenPositionWithSafePrice(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, liquidity *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(liquidity.Bytes()))
	dataField := "updateAndGetTokensForGivenPositionWithSafePrice" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) UpdateAndGetSafePrice(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, input EsdtTokenPayment) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(input.Token_identifier)))
	bytes164 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes164, input.Token_nonce)
	_args = append(_args, hex.EncodeToString(bytes164))
	_args = append(_args, hex.EncodeToString(input.Amount.Bytes()))
	dataField := "updateAndGetSafePrice" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetMaxObservationsPerRecord(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, max_observations_per_record uint64) error {
	_args := make([]string, 0)
	bytes064 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes064, max_observations_per_record)
	_args = append(_args, hex.EncodeToString(bytes064))
	dataField := "setMaxObservationsPerRecord" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetLockingDeadlineEpoch(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, new_deadline uint64) error {
	_args := make([]string, 0)
	bytes064 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes064, new_deadline)
	_args = append(_args, hex.EncodeToString(bytes064))
	dataField := "setLockingDeadlineEpoch" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetLockingScAddress(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, new_address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(new_address))
	dataField := "setLockingScAddress" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) SetUnlockEpoch(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, new_epoch uint64) error {
	_args := make([]string, 0)
	bytes064 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes064, new_epoch)
	_args = append(_args, hex.EncodeToString(bytes064))
	dataField := "setUnlockEpoch" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) AddAdmin(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "addAdmin" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) RemoveAdmin(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "removeAdmin" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

// only owner
func (contract *Pair) UpdateOwnerOrAdmin(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, previous_owner Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(previous_owner))
	dataField := "updateOwnerOrAdmin" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) AddToPauseWhitelist(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address_list []Address) error {
	_args := make([]string, 0)
	for _, elem := range address_list {
		_args = append(_args, hex.EncodeToString(elem))
	}
	dataField := "addToPauseWhitelist" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) RemoveFromPauseWhitelist(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address_list []Address) error {
	_args := make([]string, 0)
	for _, elem := range address_list {
		_args = append(_args, hex.EncodeToString(elem))
	}
	dataField := "removeFromPauseWhitelist" + "@" + strings.Join(_args, "@")
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) Pause(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "pause"
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}

func (contract *Pair) Resume(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "resume"
	hash, err := contract.netMan.SendTransaction(_pk, contract.contractAddress, _value, _gasLimit, dataField, _nonce)
	if err != nil {
		return err
	}

	err = contract.netMan.GetTxResult(hash)
	if err != nil {
		return err
	}

	return nil
}
