use super::errors::BinNotFound;
use super::grid::Grid;
use ndarray::prelude::{ArrayBase, ArrayD, ArrayViewD, Axis, Ix1, Ix2};
use ndarray::Data;
use std::ops::Add;

/// Binned statistic data structure.
pub struct BinnedStatistic<A: Ord, T: num_traits::Num> {
    counts: ArrayD<usize>,
    sum: ArrayD<T>,
    grid: Grid<A>,
}

impl<A, T> BinnedStatistic<A, T>
where
    A: Ord,
    T: Clone + num_traits::Num,
{
    /// Returns a new instance of BinnedStatistic given a [`Grid`].
    ///
    /// [`Grid`]: struct.Grid.html
    pub fn new(grid: Grid<A>) -> Self {
        let counts = ArrayD::zeros(grid.shape());
        let sum = ArrayD::zeros(grid.shape());
        BinnedStatistic { counts, sum, grid }
    }

    /// Adds a single sample to the binned statistic.
    ///
    /// **Panics** if dimensions do not match: `self.ndim() != sample.len()`.
    ///
    /// # Example:
    /// ```
    /// use ndarray::array;
    /// use ndarray_stats::histogram::{Edges, Bins, BinnedStatistic, Grid};
    /// use noisy_float::types::n64;
    ///
    /// let edges = Edges::from(vec![n64(-1.), n64(0.), n64(1.)]);
    /// let bins = Bins::new(edges);
    /// let square_grid = Grid::from(vec![bins.clone(), bins.clone()]);
    /// let mut binned_statistic = BinnedStatistic::new(square_grid);
    ///
    /// let sample = array![n64(0.5), n64(0.6)];
    ///
    /// binned_statistic.add_sample(&sample, n64(1.0))?;
    /// binned_statistic.add_sample(&sample, n64(2.0))?;
    ///
    /// let binned_statistic_matrix = binned_statistic.sum();
    /// let expected = array![
    ///     [0.0, 0.0],
    ///     [0.0, 3.0],
    /// ];
    /// assert_eq!(binned_statistic_matrix, expected.into_dyn());
    /// # Ok::<(), Box<std::error::Error>>(())
    /// ```
    pub fn add_sample<S>(&mut self, sample: &ArrayBase<S, Ix1>, value: T) -> Result<(), BinNotFound>
    where
        S: Data<Elem = A>,
        T: Copy + num_traits::Num,
    {
        match self.grid.index_of(sample) {
            Some(bin_index) => {
                self.counts[&*bin_index] += 1usize;
                self.sum[&*bin_index] = self.sum[&*bin_index] + value;
                Ok(())
            }
            None => Err(BinNotFound),
        }
    }

    /// Returns the number of dimensions of the space the binned statistic is covering.
    pub fn ndim(&self) -> usize {
        debug_assert_eq!(self.counts.ndim(), self.grid.ndim());
        self.counts.ndim()
    }

    /// Borrows a view on the binned statistic `sum` matrix.
    pub fn sum(&self) -> ArrayViewD<'_, T> {
        self.sum.view()
    }

    /// Borrows a view on the binned statistic `counts` matrix (equivalent to histogram).
    pub fn counts(&self) -> ArrayViewD<'_, usize> {
        self.counts.view()
    }

    /// Borrows an immutable reference to the binned statistic grid.
    pub fn grid(&self) -> &Grid<A> {
        &self.grid
    }
}

impl<A: Ord, T: Copy + num_traits::Num + Add<Output = T>> Add for BinnedStatistic<A, T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.grid != other.grid {
            panic!("`BinnedStatistics` can only be added for the same `grid`!")
        };

        BinnedStatistic {
            counts: &self.counts + &other.counts,
            sum: &self.sum + &other.sum,
            grid: self.grid,
        }
    }
}

/// Extension trait for `ArrayBase` providing methods to compute histograms.
pub trait BinnedStatisticExt<A, S, T>
where
    S: Data<Elem = A>,
    T: Copy + num_traits::Num,
{
    /// Returns the [histogram](https://en.wikipedia.org/wiki/Histogram)
    /// for a 2-dimensional array of points `M`.
    ///
    /// Let `(n, d)` be the shape of `M`:
    /// - `n` is the number of points;
    /// - `d` is the number of dimensions of the space those points belong to.
    /// It follows that every column in `M` is a `d`-dimensional point.
    ///
    /// For example: a (3, 4) matrix `M` is a collection of 3 points in a
    /// 4-dimensional space.
    ///
    /// Important: points outside the grid are ignored!
    ///
    /// **Panics** if `d` is different from `grid.ndim()`.
    ///
    /// # Example:
    ///
    /// ```
    /// use ndarray::array;
    /// use ndarray_stats::{
    ///     HistogramExt,
    ///     histogram::{
    ///         Histogram, Grid, GridBuilder,
    ///         Edges, Bins,
    ///         strategies::Sqrt},
    /// };
    /// use noisy_float::types::{N64, n64};
    ///
    /// let observations = array![
    ///     [n64(1.), n64(0.5)],
    ///     [n64(-0.5), n64(1.)],
    ///     [n64(-1.), n64(-0.5)],
    ///     [n64(0.5), n64(-1.)]
    /// ];
    /// let grid = GridBuilder::<Sqrt<N64>>::from_array(&observations).unwrap().build();
    /// let expected_grid = Grid::from(
    ///     vec![
    ///         Bins::new(Edges::from(vec![n64(-1.), n64(0.), n64(1.), n64(2.)])),
    ///         Bins::new(Edges::from(vec![n64(-1.), n64(0.), n64(1.), n64(2.)])),
    ///     ]
    /// );
    /// assert_eq!(grid, expected_grid);
    ///
    /// let histogram = observations.histogram(grid);
    ///
    /// let histogram_matrix = histogram.counts();
    /// // Bins are left inclusive, right exclusive!
    /// let expected = array![
    ///     [1, 0, 1],
    ///     [1, 0, 0],
    ///     [0, 1, 0],
    /// ];
    /// assert_eq!(histogram_matrix, expected.into_dyn());
    /// ```
    fn binned_statistic(&self, grid: Grid<A>, values: ArrayD<T>) -> BinnedStatistic<A, T>
    where
        A: Ord;

    private_decl! {}
}

impl<A, S, T> BinnedStatisticExt<A, S, T> for ArrayBase<S, Ix2>
where
    S: Data<Elem = A>,
    A: Ord,
    T: Copy + num_traits::Num,
{
    fn binned_statistic(&self, grid: Grid<A>, values: ArrayD<T>) -> BinnedStatistic<A, T> {
        let mut binned_statistic = BinnedStatistic::new(grid);
        for (sample, value) in self.axis_iter(Axis(0)).zip(&values) {
            let _ = binned_statistic.add_sample(&sample, *value);
        }
        binned_statistic
    }

    private_impl! {}
}
