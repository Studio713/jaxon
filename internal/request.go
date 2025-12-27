package internal

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"mime/multipart"
	"net/http"
	"os"
	"path/filepath"
	"time"
)

type Product struct {
	Name            string `json:"name"`
	Description     string `json:"description"`
	ImageFile       string `json:"imageFile"`
	Price           int64  `json:"price"`
	RegionalPricing bool   `json:"isRegionalPricingEnabled"`
}

type PriceInformation struct {
	DefaultPriceInRobux int64    `json:"defaultPriceInRobux"`
	EnabledFeatures     []string `json:"enabledFeatures"`
}
type ProductResponse struct {
	ProductID        int64            `json:"productId"`
	Name             string           `json:"name"`
	Description      string           `json:"description"`
	IconImageAssetID int64            `json:"iconImageAssetId"`
	UniverseID       int64            `json:"universeId"`
	IsForSale        bool             `json:"isForSale"`
	StorePageEnabled bool             `json:"storePageEnabled"`
	PriceInformation PriceInformation `json:"priceInformation"`
	IsImmutable      bool             `json:"isImmutable"`
	CreatedTimestamp time.Time        `json:"createdTimestamp"`
	UpdatedTimestamp time.Time        `json:"updatedTimestamp"`
}

type GamepassResponse struct {
	GamepassID       int64            `json:"gamePassId"`
	Name             string           `json:"name"`
	Description      string           `json:"description"`
	IsForSale        bool             `json:"isForSale"`
	IconAssetID      int64            `json:"iconAssetId"`
	PriceInformation PriceInformation `json:"priceInformation"`
	CreatedTimestamp time.Time        `json:"createdTimestamp"`
	UpdatedTimestamp time.Time        `json:"updatedTimestamp"`
}

var RobloxApiUrl = "https://apis.roblox.com"

func getGamepassUrl() string {
	return RobloxApiUrl + fmt.Sprintf("/game-passes/v1/universes/%d/game-passes", AppConfig.Project.UniverseId)
}

func getProductUrl() string {
	return RobloxApiUrl + fmt.Sprintf("/developer-products/v2/universes/%d/developer-products", AppConfig.Project.UniverseId)
}

func getGamepassUpdateUrl(id int64) string {
	return RobloxApiUrl + fmt.Sprintf("/game-passes/v1/universes/%d/game-passes/%d", AppConfig.Project.UniverseId, id)
}

func getProductUpdateUrl(id int64) string {
	return RobloxApiUrl + fmt.Sprintf("/developer-products/v2/universes/%d/developer-products/%d", AppConfig.Project.UniverseId, id)
}

func getGamepassInfoUrl(id int64) string {
	return RobloxApiUrl + fmt.Sprintf("/game-passes/v1/universes/%d/game-passes/%d/creator", AppConfig.Project.UniverseId, id)
}

func getProductInfoUrl(id int64) string {
	return RobloxApiUrl + fmt.Sprintf("/developer-products/v2/universes/%d/developer-products/%d/creator", AppConfig.Project.UniverseId, id)
}

func writeFieldsToForm(writer multipart.Writer, data Product) {
	writer.WriteField("name", data.Name)
	writer.WriteField("description", data.Description)
	writer.WriteField("price", fmt.Sprintf("%d", data.Price))
	writer.WriteField("isForSale", "true")
	writer.WriteField("isRegionalPricingEnabled", fmt.Sprintf("%t", data.RegionalPricing))

	if data.ImageFile != "" {
		if _, err := os.Stat(data.ImageFile); err == nil {
			file, err := os.Open(data.ImageFile)
			if err != nil {
				log.Fatal(err)
			}
			defer file.Close()

			part, err := writer.CreateFormFile("imageFile", filepath.Base(data.ImageFile))
			if err != nil {
				log.Fatal(err)
			}
			_, err = io.Copy(part, file)
			if err != nil {
				log.Fatal(err)
			}
		} else if !os.IsNotExist(err) {
			log.Fatal(err)
		}
	}
}

func postRequest(url string, writer multipart.Writer, buf *bytes.Buffer) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodPost, url, buf)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", writer.FormDataContentType())
	req.Header.Set("x-api-key", Env.API_KEY)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func patchRequest(url string, writer multipart.Writer, buf *bytes.Buffer) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodPatch, url, buf)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", writer.FormDataContentType())
	req.Header.Set("x-api-key", Env.API_KEY)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func getRequest(url string, buf *bytes.Buffer) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodGet, url, buf)
	if err != nil {
		return nil, err
	}

	req.Header.Set("x-api-key", Env.API_KEY)

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func decodeResponse[T any](resp *http.Response) (*T, error) {
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("Request failed (%d): %s",
			resp.StatusCode,
			body)
	}

	var out T
	if err := json.NewDecoder(resp.Body).Decode(&out); err != nil {
		return nil, err
	}

	return &out, nil
}

func getProductInfo[T any](url string) (*T, error) {
	var buf bytes.Buffer

	resp, err := getRequest(url, &buf)
	if err != nil {
		return nil, err
	}

	var out T
	if err := json.NewDecoder(resp.Body).Decode(&out); err != nil {
		return nil, err
	}

	return &out, nil
}

func CreateGamepass(data Product) *GamepassResponse {
	url := getGamepassUrl()

	var buf bytes.Buffer
	writer := multipart.NewWriter(&buf)

	writeFieldsToForm(*writer, data)

	writer.Close()

	resp, err := postRequest(url, *writer, &buf)
	if err != nil {
		log.Fatal(err)
	}

	info, err := decodeResponse[GamepassResponse](resp)
	if err != nil {
		log.Fatal(err)
	}

	return info
}

func CreateProduct(data Product) *ProductResponse {
	url := getProductUrl()

	var buf bytes.Buffer
	writer := multipart.NewWriter(&buf)

	writeFieldsToForm(*writer, data)

	writer.Close()

	resp, err := postRequest(url, *writer, &buf)
	if err != nil {
		log.Fatal(err)
	}

	info, err := decodeResponse[ProductResponse](resp)
	if err != nil {
		log.Fatal(err)
	}

	return info
}

func UpdateGamepass(id int64, data Product) *GamepassResponse {
	url := getGamepassUpdateUrl(id)

	var buf bytes.Buffer
	writer := multipart.NewWriter(&buf)

	writeFieldsToForm(*writer, data)

	writer.Close()

	_, err := patchRequest(url, *writer, &buf)
	if err != nil {
		log.Fatal(err)
	}

	infoUrl := getGamepassInfoUrl(id)

	info, err := getProductInfo[GamepassResponse](infoUrl)
	if err != nil {
		log.Fatal(err)
	}

	return info
}

func UpdateProduct(id int64, data Product) *ProductResponse {
	url := getProductUpdateUrl(id)

	var buf bytes.Buffer
	writer := multipart.NewWriter(&buf)

	writeFieldsToForm(*writer, data)

	writer.Close()

	_, err := patchRequest(url, *writer, &buf)
	if err != nil {
		log.Fatal(err)
	}

	infoUrl := getProductInfoUrl(id)

	info, err := getProductInfo[ProductResponse](infoUrl)
	if err != nil {
		log.Fatal(err)
	}

	return info
}
