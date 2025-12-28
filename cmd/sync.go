package cmd

import (
	"log"
	"strconv"
	"sync"

	"github.com/Studio713/jaxon/internal"
	"github.com/spf13/cobra"
)

const maxWorkers = 5

var syncCmd = &cobra.Command{
	Use:   "sync",
	Short: "Sync local product definitions to Roblox",
	Long: `Synchronize products defined in the local products.json file
with Roblox developer products and game passes.`,
	Run: func(cmd *cobra.Command, args []string) {
		internal.LoadConfig()
		internal.LoadEnv()

		filepath := internal.AppConfig.Files.Products
		productsJson := internal.ReadJson(filepath)

		hashes := internal.GetHashes()
		passesMap := make(map[int64]internal.ProductCodeMap)
		productsMap := make(map[int64]internal.ProductCodeMap)

		var waitGroup sync.WaitGroup
		var mutex sync.Mutex

		jobs := make(chan *internal.ProductJson)

		// productList, err := internal.ListProducts()

		// workers
		log.Println("Syncing products")
		for range maxWorkers {
			go func() {
				// defer waitGroup.Done()
				// ^^^  this errors
				for product := range jobs {
					productHash := internal.GetProductHash(product)
					productStruct := internal.Product{
						Name:            product.Name,
						Description:     product.Description,
						ImageFile:       product.Image,
						Price:           product.Price,
						RegionalPricing: product.RegionalPricing,
					}

					if product.Id > 0 {
						// Has been assigned a product id
						// so we update it

						// hash is the same so it hasn't been updated
						if existingHash, exists := hashes[product.Id]; exists {
							if !(existingHash == productHash) {
								continue
							}
						}

						switch product.Type {
						case "Product":
							info := internal.UpdateProduct(product.Id, productStruct)
							mutex.Lock()
							hashes[product.Id] = productHash
							product.Id = info.ProductID
							productsMap[info.ProductID] = internal.ProductCodeMap{
								Name:  info.Name,
								Id:    info.ProductID,
								Image: "rbxassetid://" + strconv.FormatInt(info.IconImageAssetID, 10),
							}
							mutex.Unlock()
						case "Gamepass":
							info := internal.UpdateGamepass(product.Id, productStruct)
							mutex.Lock()
							hashes[product.Id] = productHash
							product.Id = info.GamepassID
							productsMap[info.GamepassID] = internal.ProductCodeMap{
								Name:  info.Name,
								Id:    info.GamepassID,
								Image: "rbxassetid://" + strconv.FormatInt(info.IconAssetID, 10),
							}
							mutex.Unlock()
						}
					} else {
						switch product.Type {
						case "Product":
							info := internal.CreateProduct(productStruct)
							mutex.Lock()
							hashes[product.Id] = productHash
							product.Id = info.ProductID
							productsMap[info.ProductID] = internal.ProductCodeMap{
								Name:  info.Name,
								Id:    info.ProductID,
								Image: "rbxassetid://" + strconv.FormatInt(info.IconImageAssetID, 10),
							}
							mutex.Unlock()

						case "Gamepass":
							info := internal.CreateGamepass(productStruct)
							mutex.Lock()
							product.Id = info.GamepassID
							hashes[product.Id] = productHash
							passesMap[info.GamepassID] = internal.ProductCodeMap{
								Name:  productStruct.Name,
								Id:    info.GamepassID,
								Image: "rbxassetid://" + strconv.FormatInt(info.IconAssetID, 10),
							}
							mutex.Unlock()
						}
					}
					waitGroup.Done()
				}
			}()
		}

		// Enqueue jobs
		for i := range productsJson {
			waitGroup.Add(1)
			jobs <- &productsJson[i]
		}

		close(jobs)

		waitGroup.Wait()

		internal.WriteProductsJson(productsJson)
		internal.GenerateCode(&productsMap, &passesMap)
		internal.WriteHashesToLockfile(hashes)

		log.Println("Synced products")
	},
}

func init() {
	rootCmd.AddCommand(syncCmd)
}
