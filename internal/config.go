package internal

import (
	"log"
	"os"

	"github.com/BurntSushi/toml"
	"github.com/joho/godotenv"
)

type Project struct {
	UniverseId int64 `toml:"universe_id"`
}

type Generation struct {
	Typescript bool `toml:"typescript"`
}

type Files struct {
	Products string `toml:"product_dir"`
	Output   string `toml:"output_dir"`
	FileName string `toml:"file_name"`
}

type Config struct {
	Project    Project    `toml:"project"`
	Generation Generation `toml:"generation"`
	Files      Files      `toml:"files"`
}

type DotEnv struct {
	API_KEY string
}

const tomlFile = "jaxon.toml"

var AppConfig Config
var Env DotEnv

func LoadConfig() {
	if _, err := toml.DecodeFile("jaxon.toml", &AppConfig); err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	if AppConfig.Project.UniverseId == 0 {
		log.Fatalf("Config missing required field or invalid: universe_id")
	}
}

func LoadEnv() {
	if err := godotenv.Load(".env"); err != nil {
		log.Println("Warning: .env file not found, relying on environment variables")
	}

	APIKEY := os.Getenv("JAXON_API_KEY")
	if APIKEY == "" {
		log.Fatal("JAXON_API_KEY not set in .env or environment")
	}
	Env.API_KEY = APIKEY
}

func InitToml() error {
	if _, err := os.Stat(tomlFile); err == nil {
		log.Println("jaxon.toml already exists")
	}

	config := Config{}

	data, err := toml.Marshal(config)
	if err != nil {
		return err
	}
	return os.WriteFile(tomlFile, data, 0644)
}
