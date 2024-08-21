use crate::Result;

/// HTTPGetter provides a way to retrieve data through HTTP GET requests
// pub trait HTTPGetter {
//   fn get(&self, uri: &http::Uri) -> http::Result<http::Response<()>>;
// }

/// Getter provides a way to retrieve data from a URI
pub trait Getter: std::fmt::Debug {
  fn get(&self, uri: &str) -> Result<Vec<u8>>;
}

// type GetterFn = dyn Fn(&str) -> Result<Vec<u8>>;

// impl Getter for GetterFn {
//   fn get(&self, uri: &str) -> Result<Vec<u8>> {
//     self(uri)
//   }
// }

// impl Getter for dyn HTTPGetter {
//   fn get(&self, uri: &str) -> Result<Vec<u8>> {
//     let _parsed_uri = uri.parse::<http::Uri>()?;

//     Err(crate::NotImplementedError("aaa").into())
//   }
// }

// type GetterFn func(uri string) ([]byte, error)

// func (fn GetterFn) Get(uri string) ([]byte, error) {
// 	return fn(uri)
// }

// func HTTPGetterAsGetter(getter HTTPGetter) Getter {
// 	return GetterFn(func(uri string) ([]byte, error) {
// 		res, err := getter.Get(uri)
// 		if err != nil {
// 			return nil, fmt.Errorf("failed to fetch remote JSON: %w", err)
// 		}
// 		defer res.Body.Close()

// 		if res.StatusCode != http.StatusOK {
// 			return nil, fmt.Errorf("provider data not found")
// 		}

// 		return io.ReadAll(res.Body)
// 	})
// }

impl Getter for reqwest::Client {
  #[tracing::instrument(skip(self), ret, err)]
  fn get(&self, uri: &str) -> Result<Vec<u8>> {
    let ret: Result<Vec<u8>> = futures::executor::block_on(async {
      let req = self.get(uri).build()?;
      let res = self.execute(req).await?;
      let data = res.bytes().await?;
      Ok(data.to_vec())
    });

    ret
  }
}

impl Getter for reqwest::blocking::Client {
  #[tracing::instrument(skip(self), ret, err)]
  fn get(&self, uri: &str) -> Result<Vec<u8>> {
    let req = self.get(uri).build()?;
    let res = self.execute(req)?;
    let data = res.bytes()?;
    Ok(data.to_vec())
  }
}
