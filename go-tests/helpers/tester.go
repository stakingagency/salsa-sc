package helpers

func StartTesting() error {
	// if err := addReserve(1); err != nil {
	// if err := removeReserve("1000000000000000000000"); err != nil {
	if err := takeLpProfit(); err != nil {
		return err
	}

	return nil
}
