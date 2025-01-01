#[cfg(feature = "async")]
mod sealed {
    use anyhow::Result;
    use async_trait::async_trait;
    use futures::StreamExt;
    use par_dfs::r#async::{Node, NodeStream};
    use std::fs::FileType;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::fs;
    use tokio_stream::wrappers::ReadDirStream;

    #[derive(Hash, PartialEq, Eq, Clone, Debug)]
    pub enum FsNode {
        File(PathBuf),
        Dir(PathBuf),
    }

    impl FsNode {
        pub fn from_type<P: Into<PathBuf>>(path: P, file_type: FileType) -> Result<Self> {
            let path = path.into();
            if file_type.is_dir() {
                Ok(Self::Dir(path))
            } else if file_type.is_file() {
                Ok(Self::File(path))
            } else {
                Err(anyhow::anyhow!(
                    "bad file type {:?} for {}",
                    file_type,
                    path.to_string_lossy()
                ))
            }
        }
    }

    impl TryFrom<PathBuf> for FsNode {
        type Error = anyhow::Error;

        fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
            let file_type = path.metadata()?.file_type();
            Self::from_type(path, file_type)
        }
    }

    #[async_trait]
    impl Node for FsNode {
        type Error = anyhow::Error;

        async fn children(
            self: Arc<Self>,
            _depth: usize,
        ) -> Result<NodeStream<Self, Self::Error>, Self::Error> {
            let children = match self.as_ref() {
                FsNode::File(_) => {
                    // no children
                    futures::stream::empty().boxed()
                }
                FsNode::Dir(path) => {
                    let path: PathBuf = path.clone();
                    // get stream of files
                    let entries = fs::read_dir(&path).await?;
                    let entries_stream = ReadDirStream::new(entries);
                    // create new nodes from the children
                    entries_stream
                        .then(move |entry| async move {
                            let entry = entry?;
                            let file_type = entry.file_type().await?;
                            Self::from_type(entry.path(), file_type)
                        })
                        .boxed()
                }
            };
            Ok(Box::pin(children.boxed()))
        }
    }
}

#[cfg(not(feature = "async"))]
fn main() {
    panic!("Feature \"async\" must be enabled for this example");
}

#[cfg(feature = "async")]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    use clap::Parser;
    use futures::StreamExt;
    use par_dfs::r#async::Bfs;
    use sealed::FsNode;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::sync::Mutex;

    #[derive(Parser, Debug)]
    pub struct Options {
        #[clap(short = 'p', long = "path", help = "path from which to iterate")]
        path: PathBuf,
        #[clap(short = 'd', long = "depth", help = "max depth", default_value = "2")]
        max_depth: usize,
    }

    #[derive(Debug, Default)]
    struct Stats {
        files: usize,
        dirs: usize,
        errs: usize,
    }

    let start = Instant::now();
    let options = Options::parse();
    let root: FsNode = options.path.try_into()?;
    let bfs: Bfs<FsNode> = Bfs::new(root, options.max_depth, true);

    let stats = Arc::new(Mutex::new(Stats::default()));

    bfs.for_each_concurrent(None, |node| {
        let stats = stats.clone();
        async move {
            println!("{node:?}");
            let mut stats = stats.lock().await;
            match node {
                Ok(FsNode::Dir(_)) => stats.dirs += 1,
                Ok(FsNode::File(_)) => stats.files += 1,
                Err(_) => stats.errs += 1,
            };
        }
    })
    .await;
    println!("found {:?} in {:?}", *stats.lock().await, start.elapsed());
    Ok(())
}
