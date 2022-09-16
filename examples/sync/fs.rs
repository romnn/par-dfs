#[cfg(feature = "sync")]
mod sealed {
    use anyhow::Result;
    use par_dfs::r#sync::{ExtendQueue, FastNode};
    use std::fs::FileType;
    use std::path::PathBuf;

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

    impl FastNode for FsNode {
        type Error = anyhow::Error;

        fn add_children<E>(&self, _depth: usize, queue: &mut E) -> Result<(), Self::Error>
        where
            E: ExtendQueue<Self, Self::Error>,
        {
            match self {
                FsNode::Dir(path) => {
                    let nodes = path.read_dir()?.map(|entry| match entry {
                        Ok(entry) => entry.path().try_into(),
                        Err(err) => Err(err.into()),
                    });
                    queue.add_all(nodes);
                }
                FsNode::File(_) => {}
            };
            Ok(())
        }
    }
}

#[cfg(not(feature = "sync"))]
fn main() {
    panic!("Feature \"sync\" must be enabled for this example");
}

#[cfg(feature = "sync")]
fn main() -> anyhow::Result<()> {
    use clap::Parser;
    use par_dfs::r#sync::FastBfs;
    #[cfg(feature = "rayon")]
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use sealed::FsNode;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Instant;

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
    let bfs: FastBfs<FsNode> = FastBfs::new(root, options.max_depth, true);

    #[cfg(feature = "rayon")]
    let bfs = bfs.into_par_iter();

    let stats = Mutex::new(Stats::default());

    bfs.for_each(|node| {
        println!("{:?}", node);
        let mut stats = stats.lock().unwrap();
        match node {
            Ok(FsNode::Dir(_)) => stats.dirs += 1,
            Ok(FsNode::File(_)) => stats.files += 1,
            Err(_) => stats.errs += 1,
        };
    });
    println!(
        "found {:?} in {:?}",
        *stats.lock().unwrap(),
        start.elapsed()
    );
    Ok(())
}
