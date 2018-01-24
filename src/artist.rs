use serde::de::{Deserialize, Deserializer};
use serde_json;

use error::*;
use query::Query;
use sunk::Sunk;
use util::*;

use album;

#[derive(Debug)]
pub struct Artist {
    pub id: u64,
    pub name: String,
    cover_id: Option<String>,
    albums: Vec<album::Album>,
    pub album_count: u64,
}

#[derive(Debug)]
pub struct ArtistInfo {
    biography: String,
    musicbrainz_id: String,
    lastfm_url: String,
    image_urls: (String, String, String),
    similar_artists: Vec<(usize, String)>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ArtistInfoSerde {
    biography: String,
    musicBrainzId: String,
    lastFmUrl: String,
    smallImageUrl: String,
    mediumImageUrl: String,
    largeImageUrl: String,
    similarArtist: Vec<SimilarArtistSerde>,
}

#[derive(Debug, Deserialize)]
struct SimilarArtistSerde {
    id: String,
    name: String,
}

impl Artist {
    pub fn albums(&self, sunk: &mut Sunk) -> Result<Vec<album::Album>> {
        if self.albums.len() as u64 != self.album_count {
            Ok(get_artist(sunk, self.id)?.albums)
        } else {
            Ok(self.albums.clone())
        }
    }

    pub fn info(
        &self,
        sunk: &mut Sunk,
        count: Option<usize>,
        include_not_present: Option<bool>,
    ) -> Result<ArtistInfo> {
        let args = Query::with("id", self.id.to_string())
            .maybe_arg("count", map_str(count))
            .maybe_arg("includeNotPresent", map_str(include_not_present))
            .build();
        let res = sunk.get("getArtistInfo", args)?;

        let serde: ArtistInfoSerde = serde_json::from_value(res)?;
        Ok(ArtistInfo {
            biography: serde.biography,
            musicbrainz_id: serde.musicBrainzId,
            lastfm_url: serde.lastFmUrl,
            image_urls: (
                serde.smallImageUrl,
                serde.mediumImageUrl,
                serde.largeImageUrl,
            ),
            similar_artists: serde
                .similarArtist
                .iter()
                .map(|a| (a.id.parse().unwrap(), a.name.to_string()))
                .collect(),
        })
    }

    impl_cover_art!();
}

impl<'de> Deserialize<'de> for Artist {
    fn deserialize<D>(de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Artist {
            id: String,
            name: String,
            cover_art: Option<String>,
            album_count: u64,
            #[serde(default)]
            album: Vec<album::Album>,
        }

        let raw = _Artist::deserialize(de)?;

        Ok(Artist {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            cover_id: raw.cover_art,
            album_count: raw.album_count,
            albums: raw.album,
        })
    }
}

pub fn get_artist(sunk: &mut Sunk, id: u64) -> Result<Artist> {
    let res = sunk.get("getArtist", Query::with("id", id))?;
    Ok(serde_json::from_value::<Artist>(res)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util;

    #[test]
    fn parse_artist() {
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();

        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.name, String::from("Misteur Valaire"));
        assert_eq!(parsed.album_count, 1);
    }

    #[test]
    fn parse_artist_deep() {
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();

        assert_eq!(parsed.albums.len() as u64, parsed.album_count);
        assert_eq!(parsed.albums[0].id, 1);
        assert_eq!(parsed.albums[0].name, String::from("Bellevue"));
        assert_eq!(parsed.albums[0].song_count, 9);
    }

    #[test]
    fn remote_artist_album_list() {
        let mut srv = test_util::demo_site().unwrap();
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();
        let albums = parsed.albums(&mut srv).unwrap();

        assert_eq!(albums[0].id, 1);
        assert_eq!(albums[0].name, String::from("Bellevue"));
        assert_eq!(albums[0].song_count, 9);
    }

    #[test]
    fn remote_artist_cover_art() {
        let mut srv = test_util::demo_site().unwrap();
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();
        assert_eq!(parsed.cover_id, Some(String::from("ar-1")));

        let cover = parsed.cover_art(&mut srv, None).unwrap();
        println!("{:?}", cover);
        assert!(!cover.is_empty())
    }

    fn raw() -> serde_json::Value {
        json!({
            "id" : "1",
            "name" : "Misteur Valaire",
            "coverArt" : "ar-1",
            "albumCount" : 1,
            "album" : [ {
                "id" : "1",
                "name" : "Bellevue",
                "artist" : "Misteur Valaire",
                "artistId" : "1",
                "coverArt" : "al-1",
                "songCount" : 9,
                "duration" : 1920,
                "playCount" : 2223,
                "created" : "2017-03-12T11:07:25.000Z",
                "genre" : "(255)"
            } ]
        })
    }

}
