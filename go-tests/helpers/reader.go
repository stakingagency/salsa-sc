package helpers

import (
	"encoding/hex"
	"math/big"
)

func readRawKey(key string) ([]byte, error) {
	hexKey := hex.EncodeToString([]byte(key))

	return scAccount.GetAccountKey(hexKey)
}

func readStringKey(key string) (string, error) {
	res, err := readRawKey(key)
	if err != nil {
		return "", err
	}

	return string(res), nil
}

func readBigIntKey(key string) (*big.Int, error) {
	res, err := readRawKey(key)
	if err != nil {
		return nil, err
	}

	return big.NewInt(0).SetBytes(res), nil
}

func readU64Key(key string) (uint64, error) {
	res, err := readBigIntKey(key)
	if err != nil {
		return 0, err
	}

	return res.Uint64(), nil
}

func keyExists(key string) bool {
	res, err := readRawKey(key)

	return err == nil && len(res) > 0
}
