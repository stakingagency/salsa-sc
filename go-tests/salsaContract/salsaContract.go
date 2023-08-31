package salsaContract

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

type Knight struct {
	Address Address
	State   KnightState
}

type Undelegation struct {
	Amount       *big.Int
	Unbond_epoch uint64
}

type UserInfo struct {
	Undelegations []Undelegation
	Reserve       *big.Int
	Delegation    *big.Int
	Knight        Address
	Heir          Address
}

type EsdtTokenPayment struct {
	Token_identifier TokenIdentifier
	Token_nonce      uint64
	Amount           *big.Int
}

type Heir struct {
	Address             Address
	Inheritance_epochs  uint64
	Last_accessed_epoch uint64
}

type KnightState int

const (
	InactiveKnight      KnightState = 0
	PendingConfirmation KnightState = 1
	ActiveKnight        KnightState = 2
)

type State int

const (
	Inactive State = 0
	Active   State = 1
)

type SalsaContract struct {
	netMan          *network.NetworkManager
	contractAddress string
}

func NewSalsaContract(contractAddress string, proxyAddress string, indexAddress string) (*SalsaContract, error) {
	netMan, err := network.NewNetworkManager(proxyAddress, indexAddress)
	if err != nil {
		return nil, err
	}

	contract := &SalsaContract{
		netMan:          netMan,
		contractAddress: contractAddress,
	}

	return contract, nil
}

func (contract *SalsaContract) GetNetworkManager() *network.NetworkManager {
	return contract.netMan
}
func (contract *SalsaContract) GetLiquidTokenId() (TokenIdentifier, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getLiquidTokenId", nil)
	if err != nil {
		return "", err
	}

	res0 := TokenIdentifier(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetLiquidTokenSupply() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getLiquidTokenSupply", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetState() (State, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getState", nil)
	if err != nil {
		return 0, err
	}

	res0 := State(big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64())

	return res0, nil
}

func (contract *SalsaContract) GetProviderAddress() (Address, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getProviderAddress", nil)
	if err != nil {
		return nil, err
	}

	res0 := res.Data.ReturnData[0]

	return res0, nil
}

func (contract *SalsaContract) GetUnbondPeriod() (uint64, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUnbondPeriod", nil)
	if err != nil {
		return 0, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64()

	return res0, nil
}

func (contract *SalsaContract) GetUserUndelegations(user Address) ([]Undelegation, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUserUndelegations", _args)
	if err != nil {
		return nil, err
	}

	res0 := make([]Undelegation, 0)
	for i := 0; i < len(res.Data.ReturnData); i++ {
		idx := 0
		ok, allOk := true, true
		_Amount, idx, ok := utils.ParseBigInt(res.Data.ReturnData[i], idx)
		allOk = allOk && ok
		_Unbond_epoch, idx, ok := utils.ParseUint64(res.Data.ReturnData[i], idx)
		allOk = allOk && ok
		if !allOk {
			return nil, errors.New("invalid response")
		}

		_item := Undelegation{
			Amount:       _Amount,
			Unbond_epoch: _Unbond_epoch,
		}
		res0 = append(res0, _item)
	}

	return res0, nil
}

func (contract *SalsaContract) GetTotalEgldStaked() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTotalEgldStaked", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetUserWithdrawnEgld() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUserWithdrawnEgld", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetTotalWithdrawnEgld() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTotalWithdrawnEgld", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetTotalUserUndelegations() ([]Undelegation, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTotalUserUndelegations", nil)
	if err != nil {
		return nil, err
	}

	res0 := make([]Undelegation, 0)
	for i := 0; i < len(res.Data.ReturnData); i++ {
		idx := 0
		ok, allOk := true, true
		_Amount, idx, ok := utils.ParseBigInt(res.Data.ReturnData[i], idx)
		allOk = allOk && ok
		_Unbond_epoch, idx, ok := utils.ParseUint64(res.Data.ReturnData[i], idx)
		allOk = allOk && ok
		if !allOk {
			return nil, errors.New("invalid response")
		}

		_item := Undelegation{
			Amount:       _Amount,
			Unbond_epoch: _Unbond_epoch,
		}
		res0 = append(res0, _item)
	}

	return res0, nil
}

func (contract *SalsaContract) GetEgldReserve() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getEgldReserve", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetReservePoints() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getReservePoints", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetAvailableEgldReserve() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getAvailableEgldReserve", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetReserveUndelegations() ([]Undelegation, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getReserveUndelegations", nil)
	if err != nil {
		return nil, err
	}

	res0 := make([]Undelegation, 0)
	for i := 0; i < len(res.Data.ReturnData); i++ {
		idx := 0
		ok, allOk := true, true
		_Amount, idx, ok := utils.ParseBigInt(res.Data.ReturnData[i], idx)
		allOk = allOk && ok
		_Unbond_epoch, idx, ok := utils.ParseUint64(res.Data.ReturnData[i], idx)
		allOk = allOk && ok
		if !allOk {
			return nil, errors.New("invalid response")
		}

		_item := Undelegation{
			Amount:       _Amount,
			Unbond_epoch: _Unbond_epoch,
		}
		res0 = append(res0, _item)
	}

	return res0, nil
}

func (contract *SalsaContract) GetUsersReservePoints(user Address) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUsersReservePoints", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetUndelegateNowFee() (uint64, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUndelegateNowFee", nil)
	if err != nil {
		return 0, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64()

	return res0, nil
}

func (contract *SalsaContract) GetReservePointsAmount(egld_amount *big.Int) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(egld_amount.Bytes()))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getReservePointsAmount", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetReserveEgldAmount(points_amount *big.Int) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(points_amount.Bytes()))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getReserveEgldAmount", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetUserReserve(user Address) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUserReserve", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetTokenPrice() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getTokenPrice", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetLegldInCustody() (*big.Int, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getLegldInCustody", nil)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetUserDelegation(user Address) (*big.Int, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUserDelegation", _args)
	if err != nil {
		return nil, err
	}

	res0 := big.NewInt(0).SetBytes(res.Data.ReturnData[0])

	return res0, nil
}

func (contract *SalsaContract) GetUserKnight(user Address) (Knight, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUserKnight", _args)
	if err != nil {
		return Knight{}, err
	}

	idx := 0
	ok, allOk := true, true
	_Address, idx, ok := utils.ParsePubkey(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_State, idx, ok := utils.ParseByte(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	if !allOk {
		return Knight{}, errors.New("invalid response")
	}

	res0 := Knight{
		Address: Address(_Address),
		State:   KnightState(_State),
	}

	return res0, nil
}

func (contract *SalsaContract) GetKnightUsers(knight Address) ([]Address, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(knight))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getKnightUsers", _args)
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

func (contract *SalsaContract) GetUserHeir(user Address) (Heir, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUserHeir", _args)
	if err != nil {
		return Heir{}, err
	}

	idx := 0
	ok, allOk := true, true
	_Address, idx, ok := utils.ParsePubkey(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_Inheritance_epochs, idx, ok := utils.ParseUint64(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_Last_accessed_epoch, idx, ok := utils.ParseUint64(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	if !allOk {
		return Heir{}, errors.New("invalid response")
	}

	res0 := Heir{
		Address:             Address(_Address),
		Inheritance_epochs:  _Inheritance_epochs,
		Last_accessed_epoch: _Last_accessed_epoch,
	}

	return res0, nil
}

func (contract *SalsaContract) GetHeirUsers(heir Address) ([]Address, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(heir))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getHeirUsers", _args)
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

func (contract *SalsaContract) GetUserInfo(user Address) (UserInfo, error) {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getUserInfo", _args)
	if err != nil {
		return UserInfo{}, err
	}

	idx := 0
	ok, allOk := true, true
	_Undelegations := make([]Undelegation, 0)
	var _len uint32
	_len, idx, ok = utils.ParseUint32(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	for l := uint32(0); l < _len; l++ {
		var _Amount *big.Int
		var _Unbond_epoch uint64
		_Amount, idx, ok = utils.ParseBigInt(res.Data.ReturnData[0], idx)
		allOk = allOk && ok
		_Unbond_epoch, idx, ok = utils.ParseUint64(res.Data.ReturnData[0], idx)
		allOk = allOk && ok
		if !allOk {
			return UserInfo{}, errors.New("invalid response")
		}

		item := Undelegation{
			Amount:       _Amount,
			Unbond_epoch: _Unbond_epoch,
		}
		_Undelegations = append(_Undelegations, item)
	}
	_Reserve, idx, ok := utils.ParseBigInt(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_Delegation, idx, ok := utils.ParseBigInt(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_Knight, idx, ok := utils.ParsePubkey(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	_Heir, idx, ok := utils.ParsePubkey(res.Data.ReturnData[0], idx)
	allOk = allOk && ok
	if !allOk {
		return UserInfo{}, errors.New("invalid response")
	}

	res0 := UserInfo{
		Undelegations: _Undelegations,
		Reserve:       _Reserve,
		Delegation:    _Delegation,
		Knight:        Address(_Knight),
		Heir:          Address(_Heir),
	}

	return res0, nil
}

func (contract *SalsaContract) GetArbitrageState() (State, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getArbitrageState", nil)
	if err != nil {
		return 0, err
	}

	res0 := State(big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64())

	return res0, nil
}

func (contract *SalsaContract) GetOnedexArbitrageState() (State, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getOnedexArbitrageState", nil)
	if err != nil {
		return 0, err
	}

	res0 := State(big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64())

	return res0, nil
}

func (contract *SalsaContract) GetXexchangeArbitrageState() (State, error) {
	res, err := contract.netMan.QuerySC(contract.contractAddress, "getXexchangeArbitrageState", nil)
	if err != nil {
		return 0, err
	}

	res0 := State(big.NewInt(0).SetBytes(res.Data.ReturnData[0]).Uint64())

	return res0, nil
}

func (contract *SalsaContract) Delegate(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, with_custody bool) error {
	_args := make([]string, 0)
	if with_custody {
		_args = append(_args, "01")
	} else {
		_args = append(_args, "00")
	}
	dataField := "delegate" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegate(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, undelegate_amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(undelegate_amount.Bytes()))
	dataField := hex.EncodeToString([]byte("unDelegate")) + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) Withdraw(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "withdraw"
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

func (contract *SalsaContract) AddToCustody(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := hex.EncodeToString([]byte("addToCustody"))
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

func (contract *SalsaContract) RemoveFromCustody(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(amount.Bytes()))
	dataField := "removeFromCustody" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) AddReserve(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "addReserve"
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

func (contract *SalsaContract) RemoveReserve(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(amount.Bytes()))
	dataField := "removeReserve" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegateNow(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, min_amount_out *big.Int, undelegate_amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(min_amount_out.Bytes()))
	_args = append(_args, hex.EncodeToString(undelegate_amount.Bytes()))
	dataField := hex.EncodeToString([]byte("unDelegateNow")) + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegateNowCustodial(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, min_amount_out *big.Int, undelegate_amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(min_amount_out.Bytes()))
	_args = append(_args, hex.EncodeToString(undelegate_amount.Bytes()))
	dataField := "unDelegateNow" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegateKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address, undelegate_amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	_args = append(_args, hex.EncodeToString(undelegate_amount.Bytes()))
	dataField := "unDelegateKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegateNowKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address, min_amount_out *big.Int, undelegate_amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	_args = append(_args, hex.EncodeToString(min_amount_out.Bytes()))
	_args = append(_args, hex.EncodeToString(undelegate_amount.Bytes()))
	dataField := "unDelegateNowKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) WithdrawKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	dataField := "withdrawKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) RemoveReserveKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address, amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	_args = append(_args, hex.EncodeToString(amount.Bytes()))
	dataField := "removeReserveKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegateHeir(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address, undelegate_amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	_args = append(_args, hex.EncodeToString(undelegate_amount.Bytes()))
	dataField := "unDelegateHeir" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegateNowHeir(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address, min_amount_out *big.Int, undelegate_amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	_args = append(_args, hex.EncodeToString(min_amount_out.Bytes()))
	_args = append(_args, hex.EncodeToString(undelegate_amount.Bytes()))
	dataField := "unDelegateNowHeir" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) WithdrawHeir(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	dataField := "withdrawHeir" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) RemoveReserveHeir(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address, amount *big.Int) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	_args = append(_args, hex.EncodeToString(amount.Bytes()))
	dataField := "removeReserveHeir" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) RegisterLiquidToken(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, token_display_name string, token_ticker string, num_decimals uint32) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString([]byte(token_display_name)))
	_args = append(_args, hex.EncodeToString([]byte(token_ticker)))
	bytes232 := make([]byte, 4)
	binary.BigEndian.PutUint32(bytes232, num_decimals)
	_args = append(_args, hex.EncodeToString(bytes232))
	dataField := "registerLiquidToken" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) SetStateActive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setStateActive"
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
func (contract *SalsaContract) SetStateInactive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setStateInactive"
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
func (contract *SalsaContract) SetProviderAddress(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "setProviderAddress" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) SetUnbondPeriod(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, period uint64) error {
	_args := make([]string, 0)
	bytes064 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes064, period)
	_args = append(_args, hex.EncodeToString(bytes064))
	dataField := "setUnbondPeriod" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) SetUndelegateNowFee(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, new_fee uint64) error {
	_args := make([]string, 0)
	bytes064 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes064, new_fee)
	_args = append(_args, hex.EncodeToString(bytes064))
	dataField := "setUndelegateNowFee" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) SetWrapSC(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "setWrapSC" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) UnDelegateAll(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "unDelegateAll"
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

func (contract *SalsaContract) Compound(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "compound"
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

func (contract *SalsaContract) WithdrawAll(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "withdrawAll"
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

func (contract *SalsaContract) ComputeWithdrawn(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "computeWithdrawn"
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
func (contract *SalsaContract) SetArbitrageActive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setArbitrageActive"
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
func (contract *SalsaContract) SetArbitrageInactive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setArbitrageInactive"
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
func (contract *SalsaContract) SetOnedexArbitrageActive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setOnedexArbitrageActive"
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
func (contract *SalsaContract) SetOnedexArbitrageInactive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setOnedexArbitrageInactive"
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
func (contract *SalsaContract) SetOnedexSC(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "setOnedexSC" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) SetOnedexPairId(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, id uint32) error {
	_args := make([]string, 0)
	bytes032 := make([]byte, 4)
	binary.BigEndian.PutUint32(bytes032, id)
	_args = append(_args, hex.EncodeToString(bytes032))
	dataField := "setOnedexPairId" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) SetXexchangeArbitrageActive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setXexchangeArbitrageActive"
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
func (contract *SalsaContract) SetXexchangeArbitrageInactive(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "setXexchangeArbitrageInactive"
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
func (contract *SalsaContract) SetXexchangeSC(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, address Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(address))
	dataField := "setXexchangeSC" + "@" + strings.Join(_args, "@")
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
func (contract *SalsaContract) TakeLpProfit(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "takeLpProfit"
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

func (contract *SalsaContract) SetKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, knight Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(knight))
	dataField := "setKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) CancelKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "cancelKnight"
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

func (contract *SalsaContract) ActivateKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "activateKnight"
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

func (contract *SalsaContract) DeactivateKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	dataField := "deactivateKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) ConfirmKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	dataField := "confirmKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) RemoveKnight(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, user Address) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(user))
	dataField := "removeKnight" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) SetHeir(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64, heir Address, inheritance_epochs uint64) error {
	_args := make([]string, 0)
	_args = append(_args, hex.EncodeToString(heir))
	bytes164 := make([]byte, 8)
	binary.BigEndian.PutUint64(bytes164, inheritance_epochs)
	_args = append(_args, hex.EncodeToString(bytes164))
	dataField := "setHeir" + "@" + strings.Join(_args, "@")
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

func (contract *SalsaContract) RemoveHeir(_pk []byte, _value float64, _gasLimit uint64, _token *data.ESDT, _nonce uint64) error {
	dataField := "removeHeir"
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
