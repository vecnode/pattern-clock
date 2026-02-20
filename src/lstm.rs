use burn::module::Module;
use burn::nn::Linear;
use burn::nn::LinearConfig;
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Configuration for LSTM model
#[derive(Debug, Clone)]
pub struct LstmConfig {
    /// Input feature dimension
    pub input_size: usize,
    /// Hidden state dimension
    pub hidden_size: usize,
    /// Number of stacked LSTM layers
    pub num_layers: usize,
    /// Whether to use bias terms
    pub bias: bool,
    /// If true, input shape is [batch, seq, features], else [seq, batch, features]
    pub batch_first: bool,
}

impl Default for LstmConfig {
    fn default() -> Self {
        Self {
            input_size: 128,
            hidden_size: 256,
            num_layers: 1,
            bias: true,
            batch_first: true,
        }
    }
}

/// LSTM Cell - processes a single timestep
#[derive(Module, Debug)]
pub struct LstmCell<B: Backend> {
    /// Input-to-hidden transformation for all gates [input, forget, cell, output]
    /// Maps input_size -> 4 * hidden_size
    gate_ih: Linear<B>,
    /// Hidden-to-hidden transformation for all gates
    /// Maps hidden_size -> 4 * hidden_size
    gate_hh: Linear<B>,
    /// Hidden dimension
    hidden_size: usize,
}

impl<B: Backend> LstmCell<B> {
    /// Create a new LSTM cell
    pub fn new(config: &LstmConfig, device: &B::Device) -> Self {
        let gate_ih = LinearConfig::new(config.input_size, 4 * config.hidden_size)
            .with_bias(config.bias)
            .init(device);
        
        let gate_hh = LinearConfig::new(config.hidden_size, 4 * config.hidden_size)
            .with_bias(config.bias)
            .init(device);

        Self {
            gate_ih,
            gate_hh,
            hidden_size: config.hidden_size,
        }
    }

    /// Forward pass for a single timestep
    /// 
    /// # Arguments
    /// * `input` - Input tensor of shape [batch_size, input_size]
    /// * `hidden` - Previous hidden state [batch_size, hidden_size]
    /// * `cell` - Previous cell state [batch_size, hidden_size]
    /// 
    /// # Returns
    /// * `(new_hidden, new_cell)` - Updated hidden and cell states
    pub fn forward(
        &self,
        input: Tensor<B, 2>,
        hidden: Tensor<B, 2>,
        cell: Tensor<B, 2>,
    ) -> (Tensor<B, 2>, Tensor<B, 2>) {
        // Compute gate activations
        let gates_ih = self.gate_ih.forward(input);
        let gates_hh = self.gate_hh.forward(hidden);
        let gates = gates_ih + gates_hh;

        // Split gates: [batch, 4*hidden] -> 4 x [batch, hidden]
        let gates_chunks = gates.chunk(4, 1);
        
        // Sigmoid: 1 / (1 + exp(-x))
        let sigmoid = |x: Tensor<B, 2>| {
            let device = x.device();
            let dims = x.dims();
            let one = Tensor::zeros(dims, &device) + 1.0;
            one.clone() / (one + (-x).exp())
        };
        
        // Tanh: (exp(2x) - 1) / (exp(2x) + 1)
        let tanh = |x: Tensor<B, 2>| {
            let device = x.device();
            let dims = x.dims();
            let two_x = x.clone() * 2.0;
            let exp_2x = two_x.exp();
            let one = Tensor::zeros(dims, &device) + 1.0;
            (exp_2x.clone() - one.clone()) / (exp_2x + one)
        };
        
        let input_gate = sigmoid(gates_chunks[0].clone());
        let forget_gate = sigmoid(gates_chunks[1].clone());
        let cell_gate = tanh(gates_chunks[2].clone());
        let output_gate = sigmoid(gates_chunks[3].clone());

        // Update cell state: c_t = f_t * c_{t-1} + i_t * g_t
        let new_cell = forget_gate * cell + input_gate * cell_gate;

        // Update hidden state: h_t = o_t * tanh(c_t)
        let new_hidden = output_gate * tanh(new_cell.clone());

        (new_hidden, new_cell)
    }
}

/// Multi-layer LSTM model
#[derive(Module, Debug)]
pub struct Lstm<B: Backend> {
    /// Stacked LSTM cells
    cells: Vec<LstmCell<B>>,
    /// Hidden state dimension
    hidden_size: usize,
    /// If true, input shape is [batch, seq, features], else [seq, batch, features]
    batch_first: bool,
}

impl<B: Backend> Lstm<B> {
    /// Create a new LSTM model
    pub fn new(config: LstmConfig, device: &B::Device) -> Self {
        let mut cells = Vec::with_capacity(config.num_layers);
        
        // First layer uses input_size, subsequent layers use hidden_size
        let mut layer_config = config.clone();
        for i in 0..config.num_layers {
            if i > 0 {
                layer_config.input_size = config.hidden_size;
            }
            cells.push(LstmCell::new(&layer_config, device));
        }

        Self { 
            cells, 
            hidden_size: config.hidden_size,
            batch_first: config.batch_first,
        }
    }

    /// Forward pass through the LSTM
    /// 
    /// # Arguments
    /// * `input` - Input sequence tensor
    ///   - If batch_first: [batch_size, seq_length, input_size]
    ///   - Otherwise: [seq_length, batch_size, input_size]
    /// * `initial_state` - Optional initial (hidden, cell) states
    /// 
    /// # Returns
    /// * `(output, final_state)` where:
    ///   - output: Same shape as input but with hidden_size in last dimension
    ///   - final_state: (hidden, cell) tensors of shape [batch_size, hidden_size]
    pub fn forward(
        &self,
        input: Tensor<B, 3>,
        initial_state: Option<(Tensor<B, 2>, Tensor<B, 2>)>,
    ) -> (Tensor<B, 3>, (Tensor<B, 2>, Tensor<B, 2>)) {
        let device = input.device();
        let [seq_len, batch_size, input_size] = if self.batch_first {
            let dims = input.dims();
            [dims[1], dims[0], dims[2]]
        } else {
            input.dims()
        };

        // Transpose if batch_first to work with [seq, batch, features]
        let mut input_seq = if self.batch_first {
            input.swap_dims(0, 1)
        } else {
            input
        };

        // Initialize states
        let (mut hidden, mut cell) = initial_state.unwrap_or_else(|| {
            (
                Tensor::zeros([batch_size, self.hidden_size], &device),
                Tensor::zeros([batch_size, self.hidden_size], &device),
            )
        });

        // Process through each layer
        let mut layer_outputs: Option<Tensor<B, 3>> = None;
        
        for layer_idx in 0..self.cells.len() {
            let mut layer_output = Vec::with_capacity(seq_len);
            
            // Reset states for each layer (except first)
            if layer_idx > 0 {
                hidden = Tensor::zeros([batch_size, self.hidden_size], &device);
                cell = Tensor::zeros([batch_size, self.hidden_size], &device);
            }

            // Process sequence
            for t in 0..seq_len {
                let input_t = input_seq.clone().slice([t..t+1, 0..batch_size, 0..input_size]).squeeze_dim(0);
                (hidden, cell) = self.cells[layer_idx].forward(input_t, hidden, cell);
                layer_output.push(hidden.clone());
            }

            // Stack outputs: [seq, batch, hidden]
            let stacked = Tensor::stack(layer_output, 0);
            
            if layer_idx < self.cells.len() - 1 {
                // Use output as input for next layer
                input_seq = stacked;
            } else {
                layer_outputs = Some(stacked);
            }
        }

        let final_output = layer_outputs.unwrap();

        // Transpose back if batch_first
        let output = if self.batch_first {
            final_output.swap_dims(0, 1)
        } else {
            final_output
        };

        (output, (hidden, cell))
    }
}
