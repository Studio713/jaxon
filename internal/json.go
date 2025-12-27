package internal

import (
	"encoding/json"
	"io"
	"log"
	"os"
)

type ProductJson struct {
	Name            string `json:"name"`
	Description     string `json:"description"`
	Type            string `json:"type"`
	Image           string `json:"image"`
	Price           int64  `json:"price"`
	RegionalPricing bool   `json:"regionalPricing"`
	Id              int64  `json:"productId"`
}

const productFile = "products.json"

func ReadJson(filepath string) []ProductJson {
	file, err := os.Open(filepath + "products.json")
	if err != nil {
		log.Fatalf("Failed to open JSON file: %v", err)
	}
	defer file.Close()

	bytes, err := io.ReadAll(file)
	if err != nil {
		log.Fatalf("Failed to read JSON file: %v", err)
	}

	var products []ProductJson
	if err := json.Unmarshal(bytes, &products); err != nil {
		log.Fatalf("Failed to parse JSON: %v", err)
	}

	return products
}

func InitProductJson() error {
	if _, err := os.Stat(productFile); err == nil {
		log.Println("products.json already exists")
	}

	defaultProducts := []ProductJson{
		{
			Name:            "Example Product",
			Description:     "Example product's description",
			Type:            "Product",
			Image:           "assets/products/example.png",
			Price:           499,
			RegionalPricing: false,
		},
		{
			Name:            "Example Gamepass",
			Description:     "Example gamepass's description",
			Type:            "Gamepass",
			Image:           "assets/gamepasses/example.png",
			Price:           499,
			RegionalPricing: false,
		},
	}

	data, err := json.MarshalIndent(defaultProducts, "", "	")
	if err != nil {
		return err
	}

	return os.WriteFile(productFile, data, 0644)
}

func WriteProductsJson(products []ProductJson) error {
	// Marshal with indentation for readability
	data, err := json.MarshalIndent(products, "", "\t")
	if err != nil {
		return err
	}

	// Write to file
	if err := os.WriteFile(productFile, data, 0644); err != nil {
		return err
	}

	return nil
}
