package main

import (
	"encoding/binary"
	"math/big"
)

const (
	stringLenCap = 4
	bigIntLenCap = 4
	bytesLenCap  = 1
	pubkeyLenCap = 32
)

func parseString(bytes []byte, index int) (string, int, bool) {
	datalen := len(bytes)
	if index+stringLenCap >= datalen {
		return "", 0, false
	}

	strLen := int(big.NewInt(0).SetBytes(bytes[index : index+stringLenCap]).Uint64())
	if index+stringLenCap+int(strLen) > datalen {
		return "", 0, false
	}

	index += stringLenCap

	return string(bytes[index : index+strLen]), index + strLen, true
}

func parseByte(bytes []byte, index int) (byte, int, bool) {
	if index+1 > len(bytes) {
		return 0, 0, false
	}

	return bytes[index], index + 1, true
}

func parseBigInt(bytes []byte, index int) (*big.Int, int, bool) {
	datalen := len(bytes)
	if index+bigIntLenCap >= datalen {
		return nil, 0, false
	}

	bigIntLen := int(big.NewInt(0).SetBytes(bytes[index : index+bigIntLenCap]).Uint64())
	if index+bigIntLenCap+int(bigIntLen) > datalen {
		return nil, 0, false
	}

	index += bigIntLenCap

	return big.NewInt(0).SetBytes(bytes[index : index+bigIntLen]), index + bigIntLen, true
}

func parseUint64(bytes []byte, index int) (uint64, int, bool) {
	if index+8 > len(bytes) {
		return 0, 0, false
	}

	return binary.BigEndian.Uint64(bytes[index : index+8]), index + 8, true
}

func parseUint32(bytes []byte, index int) (uint32, int, bool) {
	if index+4 > len(bytes) {
		return 0, 0, false
	}

	return binary.BigEndian.Uint32(bytes[index : index+4]), index + 4, true
}

func parseUint16(bytes []byte, index int) (uint16, int, bool) {
	if index+2 > len(bytes) {
		return 0, 0, false
	}

	return binary.BigEndian.Uint16(bytes[index : index+2]), index + 2, true
}

func parseByteArray(bytes []byte, index int) ([]byte, int, bool) {
	datalen := len(bytes)
	if index+bytesLenCap >= datalen {
		return nil, 0, false
	}

	strLen := int(big.NewInt(0).SetBytes(bytes[index : index+bytesLenCap]).Uint64())
	if index+bytesLenCap+int(strLen) > datalen {
		return nil, 0, false
	}

	index += bytesLenCap

	return bytes[index : index+strLen], index + strLen, true
}

func parsePubkey(bytes []byte, index int) ([]byte, int, bool) {
	if index+pubkeyLenCap > len(bytes) {
		return nil, 0, false
	}

	return bytes[index : index+pubkeyLenCap], index + pubkeyLenCap, true
}