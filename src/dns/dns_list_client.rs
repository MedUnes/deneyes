use crate::dns::dns_server::DnsServer;
use crate::errors::DneyesError;
use std::collections::HashMap;

pub struct Config {
    pub base_url: &'static str,
    pub supported_countries: HashMap<&'static str, &'static str>,
}
pub struct Client {
    config: Config,
}

impl Client {
    pub fn new() -> Client {
        Client {
            config: Config::create(),
        }
    }
    pub async fn fetch_dns_server_list(
        &self,
        country_code: &str,
    ) -> Result<HashMap<String, DnsServer>, DneyesError> {
        let url = format!("{}/{}.json", self.config.base_url, country_code);
        let dns_server_list = reqwest::get(url)
            .await
            .map_err(|e| DneyesError::Dns(e.to_string()))?
            .json::<Vec<DnsServer>>()
            .await
            .map_err(|e| DneyesError::Dns(e.to_string()))?;

        Ok(dns_server_list
            .into_iter()
            .map(|dns_server| (dns_server.ip.to_string(), dns_server))
            .collect())
    }
}

impl Config {
    fn build_country_map() -> HashMap<&'static str, &'static str> {
        let country_codes = vec![
            "af", "al", "dz", "as", "ad", "ao", "aq", "ag", "ar", "am", "aw", "au", "at", "az",
            "bs", "bh", "bd", "bb", "by", "be", "bz", "bj", "bm", "bt", "bo", "bq", "ba", "bw",
            "br", "bn", "bg", "bf", "bi", "kh", "cm", "ca", "cv", "ky", "td", "cl", "cn", "co",
            "cg", "cd", "cr", "hr", "cu", "cy", "cz", "ci", "dk", "do", "ec", "eg", "sv", "gq",
            "ee", "et", "fi", "fr", "gf", "pf", "ga", "ge", "de", "gh", "gi", "gr", "gl", "gp",
            "gu", "gt", "gg", "gn", "hn", "hk", "hu", "is", "in", "id", "ir", "iq", "ie", "im",
            "il", "it", "jm", "jp", "je", "jo", "kz", "ke", "kr", "kw", "kg", "la", "lv", "lb",
            "lr", "ly", "li", "lt", "lu", "mo", "mk", "mg", "mw", "my", "mv", "ml", "mt", "mh",
            "mq", "mr", "mu", "yt", "mx", "md", "mc", "mn", "me", "ma", "mz", "mm", "na", "np",
            "nl", "nc", "nz", "ni", "ne", "ng", "no", "om", "pk", "pw", "ps", "pa", "pg", "py",
            "pe", "ph", "pl", "pt", "pr", "qa", "ro", "ru", "rw", "re", "vc", "sa", "sn", "rs",
            "sc", "sl", "sg", "sk", "si", "sb", "so", "za", "es", "lk", "sd", "sz", "se", "ch",
            "sy", "tw", "tj", "tz", "th", "tl", "tg", "tt", "tn", "tr", "ug", "ua", "ae", "gb",
            "us", "uy", "uz", "ve", "vn", "vi", "xk", "ye",
        ];

        let country_names = vec![
            "Afghanistan",
            "Albania",
            "Algeria",
            "American Samoa",
            "Andorra",
            "Angola",
            "Antarctica",
            "Antigua and Barbuda",
            "Argentina",
            "Armenia",
            "Aruba",
            "Australia",
            "Austria",
            "Azerbaijan",
            "Bahamas",
            "Bahrain",
            "Bangladesh",
            "Barbados",
            "Belarus",
            "Belgium",
            "Belize",
            "Benin",
            "Bermuda",
            "Bhutan",
            "Bolivia",
            "Bonaire, Sint Eustatius and Saba",
            "Bosnia and Herzegovina",
            "Botswana",
            "Brazil",
            "Brunei Darussalam",
            "Bulgaria",
            "Burkina Faso",
            "Burundi",
            "Cambodia",
            "Cameroon",
            "Canada",
            "Cape Verde",
            "Cayman Islands",
            "Chad",
            "Chile",
            "China",
            "Colombia",
            "Congo",
            "Congo, the Democratic Republic of the",
            "Costa Rica",
            "Croatia",
            "Cuba",
            "Cyprus",
            "Czech Republic",
            "Côte d'Ivoire",
            "Denmark",
            "Dominican Republic",
            "Ecuador",
            "Egypt",
            "El Salvador",
            "Equatorial Guinea",
            "Estonia",
            "Ethiopia",
            "Finland",
            "France",
            "French Guiana",
            "French Polynesia",
            "Gabon",
            "Georgia",
            "Germany",
            "Ghana",
            "Gibraltar",
            "Greece",
            "Greenland",
            "Guadeloupe",
            "Guam",
            "Guatemala",
            "Guernsey",
            "Guinea",
            "Honduras",
            "Hong Kong",
            "Hungary",
            "Iceland",
            "India",
            "Indonesia",
            "Iran, Islamic Republic of",
            "Iraq",
            "Ireland",
            "Isle of Man",
            "Israel",
            "Italy",
            "Jamaica",
            "Japan",
            "Jersey",
            "Jordan",
            "Kazakhstan",
            "Kenya",
            "Korea, Republic of",
            "Kuwait",
            "Kyrgyzstan",
            "Lao People's Democratic Republic",
            "Latvia",
            "Lebanon",
            "Liberia",
            "Libya",
            "Liechtenstein",
            "Lithuania",
            "Luxembourg",
            "Macao",
            "Macedonia, the former Yugoslav Republic of",
            "Madagascar",
            "Malawi",
            "Malaysia",
            "Maldives",
            "Mali",
            "Malta",
            "Marshall Islands",
            "Martinique",
            "Mauritania",
            "Mauritius",
            "Mayotte",
            "Mexico",
            "Moldova, Republic of",
            "Monaco",
            "Mongolia",
            "Montenegro",
            "Morocco",
            "Mozambique",
            "Myanmar",
            "Namibia",
            "Nepal",
            "Netherlands",
            "New Caledonia",
            "New Zealand",
            "Nicaragua",
            "Niger",
            "Nigeria",
            "Norway",
            "Oman",
            "Pakistan",
            "Palau",
            "Palestinian Territory, Occupied",
            "Panama",
            "Papua New Guinea",
            "Paraguay",
            "Peru",
            "Philippines",
            "Poland",
            "Portugal",
            "Puerto Rico",
            "Qatar",
            "Romania",
            "Russian Federation",
            "Rwanda",
            "Réunion",
            "Saint Vincent and the Grenadines",
            "Saudi Arabia",
            "Senegal",
            "Serbia",
            "Seychelles",
            "Sierra Leone",
            "Singapore",
            "Slovakia",
            "Slovenia",
            "Solomon Islands",
            "Somalia",
            "South Africa",
            "Spain",
            "Sri Lanka",
            "Sudan",
            "Swaziland",
            "Sweden",
            "Switzerland",
            "Syrian Arab Republic",
            "Taiwan, Province of China",
            "Tajikistan",
            "Tanzania, United Republic of",
            "Thailand",
            "Timor-Leste",
            "Togo",
            "Trinidad and Tobago",
            "Tunisia",
            "Turkey",
            "Turkmenistan",
            "Tuvalu",
            "Uganda",
            "Ukraine",
            "United Arab Emirates",
            "United Kingdom",
            "United States",
            "Uruguay",
            "Uzbekistan",
            "Venezuela, Bolivarian Republic of",
            "Viet Nam",
            "Virgin Islands, U.S.",
            "Kosovo",
            "Yemen",
        ];
        country_codes
            .into_iter()
            .zip(country_names.into_iter())
            .collect()
    }
    pub fn create() -> Self {
        Config {
            base_url: "https://public-dns.info/nameserver",
            supported_countries: Self::build_country_map(),
        }
    }
}

mod test {
    #[cfg(test)]
    #[test]
    fn test_config_create() {
        use super::Config;

        let config = Config::create();
        assert_eq!(config.base_url, "https://public-dns.info/nameserver");
        assert_eq!(*config.supported_countries.get("tn").unwrap(), "Tunisia");
    }

    #[tokio::test]
    async fn test_fetch_dns_server_list() {
        use super::*;

        let client = Client::new();
        let dns_server_list_us = client.fetch_dns_server_list("us").await.unwrap();
        assert_eq!(
            "8.8.8.8",
            &(dns_server_list_us.get(&"8.8.8.8".to_string()).unwrap().ip).to_string()
        );
        assert_eq!(
            "dns.google.",
            &(dns_server_list_us.get(&"8.8.8.8".to_string()).unwrap().name).to_string()
        );
    }
}
