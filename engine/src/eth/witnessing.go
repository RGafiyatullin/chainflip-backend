ddwitnessing
	any network failure needs exponential back off

	historically witnessing needs checkpointing
	
	central ingress of data

	timeouts
		exponential backoff

	what about getting consistent data

	batch rpc requests
		// only related


multiple backup endpoints
checkpointing
	clear out inactive epochs
	with witness confirmations
exponential backoff per request? per client? centrally?
reorg handling
pre-witnessing
both historically and uptodate witnessing (out of order)
		zip
multiple backup endpoints
	connection drops
		want to use other endpoint (in background)
all pending requests centrally
	allow pending requests to be delayed


requests have a closure that generates a future (independent of endpoint, and possibly batching? no)
	timeouts exponential backoff, and keep old requests
	how to handle errors from requests?
		possibly return errors
		otherwise
			exponentially backoff
			if made from, multiple requests, redo all

how to choose endpoint? lowest backoff time

decrease backoff when requests work (decrease to lowest backoff larger than time it took)???

if endpoint fails, exponentially backoff reconnecting
	Logging critical errors

Requests that are very old get deleted (get to maximum backoff/or some maximum timeout (VERY LONG))?

stream handling
	work the same way, but have an expected frequency

	need method for requests that work tospawn new requests internally

		// trait?
		// next_request

	maybe each endpoint exposes high level stream abstraction

latest state
	use polls only

	what if you use streams




some errors should be returned to call location and panic with good error message




witnessing
	checkpointing (optional)
		only if fill witnessing
	toggle on and off
	ring buffers?
		6 hours of blocks
		store on disk
		only confirmed blocks
	only treat witnessing as complete if extrinsics went through (even if they failed on-chain)
		bitmap for witnessing status
			stored on disk
	zip old and new witnessing for fill witnessers
	allow unordered witnessing
	allow concurrently witnessing epochs


GENERIC info
	block numbers (ordered)
	block hashes
	some block chains are finalized, some aren't

there maybe multiple different pieces of info we want, not just the blocks?
	should they be in separate streams?

fill witnesser
	closure that produces future
		go and do this for every block
	here is a method to know which block ranges you should be active in
	and here is when witnessig old stuff becomes pointless

	buffered witnesses
		so not working old witnesses don't block new witnesses
	stream old and new blocks fairly
	spawn for each range
		buffer stream and run concurrently the witnessing future

	witneses that fail?
		exponentially backoff
		or complete



latest witnesser
	method to tell you when to be active or not (just time wise not block wise)
	async closure to run at some frequency?
	
	if fails => rerun whole



enum SafetyLevel - Different for each chain



epoch witnesser



list witnessers
	eth

	btc (blocking)
		block witnesser
			for transactions to addresses (listen of addresses to witness, from SC)
			for broadcasts (listen of transactions to look for, from SC)
			chain tracking (inside the block witnesser)
	dot
		block witnesser
			vault rotation
			deposits
			TransactionFeePaid
			broadcasts


WitnesserClient
	// central data???
	// all witnessers can access this data
	// knows active epochs
	// 

// either central system starts witnessers or independent witnessers watch to startup

// definitely need a single system doing all the epochs instead of separate?

UnorderedStream

Range trait
	start and end
	end is given asynchronously

safety can be provided centrally
	with stream of safe blocks
fill can work by having a db of completed blocks (bitmap)
	db should have version info
	epoch based



when receiving things to witness
	// want to do so generically
	// id and item
	// stream that provides things to witness and when to stop witnessing
	// item type of data stream will then be block and the items?

fill
	reorgs
		check hash chain or use finalization

eth::witnesser()
	.blocks()
	.historical()
	.safety(5)
	.ring(1024) // you are receiving things to witnesses of, you need to check old blocks
		// and fill ring on start up

		// need to know how to get info for particular block or a stream



	.fill(...)
	.do(|...| async move {
		// ...
	})
	.start()
	.await;



StateChangeWitnesser
StatusWitnesser			


period
polling or stream
safety (reorg handling)
checkpointing

// internal iteration

// these are more like data streams

// data stream may or may not include historical data
	// meaning goes back to epoch start and checkpoints


// data stream of epochs
	// the item types is the start and a future that resolves to the end point

// checkpointing needs
// stream of oldest active epoch
// stream of newest active epoch
	// use cached streams
// some data streams need to be ordered
// 
 
// witnessers should receive a handle to give extrinsic calls to and it can handle if you want to finalize or not

witness(|.|


)


// limit witnesses up to maximum point via some other input
	// also consider reorgs will this behaviour causes more problems

// block witnessing past a particular point
	// ideally don't buffer pending blocks


backfill
	db of witnessed

ring buffer doesn't work well with unordered blocks

could have idea of data sources
	you can make them frm a central process
	and then make further witnessers from them

DataSource
	periodic()
	blocks()

trait DataSource {
	async run(&mut self, async closure);

	async run_single(&mut self, async closure);
}

or some kind of Stream like trait

run(|epoch, stream| -> stream)

trait DataSource {
	fn run(self, |epoch, stream| -> ()) // streams are of generic wrap types that impl a trait BlockAssociated { fn get_block_number() }

	
}


pub trait WitnesserSource {
	type Stream<'a>;

	async fn run<for<'a> F: FnMut(Epoch, Self::Stream<'a>) -> LocalFuture<'a>>(f: F)
}



safety
buffering
retrying - checkpointing
ring buffer?
limit witness to particular block height
finalize or not

// need to communicate epoch ends and inactive epochs


// safety and checkpointing

safety




safety
retrying with db

chain tracking
block witnessing at max of our block time or external block time
	every block or not

witness pacing (simply slow blocks until the chain tracking reachs the given block number)

for chain tracking you don't care about reorgs, only the latest block

could allow for multiple sources per chain?

ExternalDataSource {
	MonotonicDataIdentifier
	UniqueDataIdentifier
	stream just outputs everything

	other method to tell system of reorgs



}

if it is a block we don't expect?
	go back and check for start of wrong hashes


assme db is all one consistent sequence of blocks

must linearly search backwards
	for example on start up go back until a hash we have seen/is in the db
		remove later blocks from db as unwitnessed
	blocks should contain their parent

		what if I want to skip blocks

	

pre-witnessing no safety or retrying?

trait ExternalChain {
	type BlockNumber: Ord + Copy + Next
	type BlockHash: Eq
	type BlockData: Clone?

	block_stream
	block

	type SafetyLevel
	safety or block_stream_with_safety (not really enough for retrying)

	if you don't need to get all history in some cases, as for example addresses timeout


	need some method to get range of blocks

	if database cannot be loaded don't get back (only go back a small fixed number of blocks) - base it on time



}

cfe expires addresses

historical witness will work by getting balance at start and at end (and accounting for fetches inbetween)

Even group blocks is weird because people may decide grouping differently
	could force consistent grouping
		cycle through all grouping getting balances, accounting for fetches

need architecture for redoing things
	and storing those that are done
	
	have a identifier for each thing
	async closure for what you want done
	and a source of identifiers
	system creates a stream of ok items (output of async closure)


	loop_select
		source (add to db)
		db_of_completed, db interally stores completed only, wrapper remembers number of retries
		also want a validation on starting (for reorgs)

	also need method to say something we did was wrong and needs redoing (part of validation)
		so what we are doing much be generated

	also need method to stop the system 

	want to optionally allow multiple retries to exist at the same time
		how to decide when this happens
			Timeouts?
			Also manually?

	also need method to fill gap between stream and existing database

BlockWitnesser

	






chain tracking
	only allow witnessing ahead of curent chain tracking block
	then use the 1/2 median

	CHECK IF WE ALREADY DO THIS

	This also makes chain tracking much easier

	Also needs us to remove chain state roleback

SC Client allow dropping requests futures to cancel internals

Going back to check for reorgs doesn't need to stale witnesser progress




type BlockNumber
type BlockHash
type BlockHeader
	BlockNumber
	BlockHash
	ParentBlockHash
fn block_header_by_number
fn block_stream

what are the costs of reorg protection?
	sometimes we don't need this, so can we optionally disable


safety is basically a stream::then and a stream::filter

"regularize" block stream - make consistent back to some point before start point? (and with a epoch back to the first block of the epoch)
	if you want to us a db, and on startup have consistently bkac to he start of the epoch?



so retrying can basically say I've seen the same block number multiple times from the stream, so I'll retry that block

remember sequence of hashes
receive new hash and number
check fits
if it doesn't
	mark blocks as needing rewitnessing
	(don't need to do this) can rely on input stream?
		not in all cases
			for example gaps due to restarting


			task that is constantly told the block and hash to linearize from
				minimum?

				the job of the task is to mark all blocks that need redoing
					needs to run concurrently
				


block sequence (BTreeMap) - Doesn't need to remember the hashes for the full length (Maybe also store number of retries???)
todo blocks (UnorderedFutures)
running blocks (UnorderedFutures) ? (merge with todo blocks) ?

loop_Select
	receive block haeder
		if fits sequence good
			add to sequence (as not yet done)
		else
			reset blocks in sequence after the block (totally remove)
			set search head block
	if head block option set => look for previous block
		when completes
			reset block in sequence
		probably what to look forwards?
			work to less like to be overwritten

	This system doesn't need to care about timeouts!

	How should it prioritize todo blocks?
		if no retries, immediately? seems bad

		for new blocks immediately, with buffering
		for retries backed off
		for old blocks with no retries
			fairly with new blocks
			maybe maximum number of old blocks concurrently


	retry - will output completed ok results

regularlize block sequence


how to retry blocks?


Central Stream
And create per epoch ones off of it
per epoch stream doesn't end, just filters


system to take a stream and make per-epoch streams
	scc


cloneable adapter?


struct Block<Payload> {
	number,
	hash,
	payload,
}

type BlockHeader = Block<()>


// decomposes into a handle and a stream?
trait BlockSource: Stream { // on the stream???
	type BlockNumber;
	type BlockHash;
	// type BlockPayload; // Not in the trait part of the stream type
		// could be put into the trait not really a problem


	header_from_number()
	header_stream()
}

adaptors are auto started?
avoid manually running the system

stream of (epoch, and block stream)



cloneable



reorgs and balance changes

pair window of blocks - 
	if we cannot get balance of at block hash
		the incoming stream needs to guarantee reorgs are visible?

// go back or not??? on a reorg
	// even if you don't go back, any failed cases should get replaced?
// can existing design work in both cases?
// how to generate block pair stream



// reconnecting may cause problems with missing reorgs (or parts of them)?
	// retry system can handle that
	// so not to put difficult expectations on underlying stream

// would be nice to cancel on going requests?
// would need list of handles


retry(retry_previous_on_reorg, )



wrapper produces underlying DataChain
chain client needs to expose
	Number and Hash types

	header_stream
	header_from_number

	idea of safe block stream (just a block number)

LagSafety impl'd on top of 

client doesn't want to run in the background all the time
	only when there is an epoch

client provides a stream (which can easily be disabled when not needed)
	stream contains an Arc to some kind of handle

run_with_out_stream





trait FinalizedDataChainClient
trait ProbablisticDataChainClient

Wrappers to DataChainClient
	Impl singular shared stream
	Regularizes stream? (fills gaps)

	Adds epoch accessors?
		and ability to get epoch/datachain stream (DataChain object)
	Pairs up windows of blocks

DataChain
	Index
	Hash

Epoch processing for an external chain should be centralised!
	so you use a central client to turn DataChainSource into a epoch/data stream composer?

// how to handle the index type being different than the epoch start and ends?


trait DataChainSource
	type Index
	type Hash
	type Stream (Include Data type)

	type header_at_index
	type header_stream

trait FinalizedDataChainSource: DataChainSource
	fn finalized_header_stream

trait ProbabilisticDataChainSource: DataChainSource

trait DataChainSafeSource: DataChainSource
	type SafetyLevel

	fn header_stream_with_safety(SafetyLevel)
	
Wrappers
	FinalizeSafety
	ProbabilisticSafety
	SingularStream
	SingularSafeStream
	FillSparseStream (concurrent requests?)
	PairWindowStream
		use where to impl both for safe and unsafe streams in single trait impl

		care must be taken here to ensure you don't fuck up the epoch block ranges

		pair is effectively a previous block and the current block
			so you are conceptually looking at the changes in the current block (second in the pair)
		
	Combine two DataChains into an unsafe and a safe DataChainSource
	Cloneable
	Map/Then (Before Pair Window Wrapper)
	Buffer?

EpochWrapper
	needs?
		Stream of Epoch starts with block Starts
		And Stream of Epoch ends with block ends
		access to active epochs start and end blocks

EpochDataChainSource


Primary problem is dealing with failures when requesting data associated with a block
	i.e. the balances of accounts

Secondary problem is correlating events to blocks (for the fetch events)


// single epoch witnesser / epoch monitor
	// given a data chain
		// give me a


task monitoring for new epochs

task/s listening for epochs and making data chain streams and running them?



one task for each epoch that has a stream and runs through it?


using wrappers for data chain is easy

epoch witnessers don't end the streams even when the stream reaches end of epoch
epoch witnessers start all active epochs

on epoch spawn a thing?
systems need tobe able to specify start, per epoch, and per stream element behaviour


on epoch run some code?
	spawn task // give the start code access to the underlying object (by doing work outside of task)
		make a data chain
		filter data past epoch end
		end stream on epoch inactive
		make a epoch data chain (adaptors that need to know the epoch, and expect epoch end)
		run the stream

need to specify
- method to make data chain
- method to make epoch data chain

// why not wrap the data chain directly

so interface is fn epoch_data_chain(epoch, end_receiver, runner) -> Stream


// scoping can be more flexible by using channels instead of generic async closures
// spawn_epoch_witnesser(scope, ...) function on the epoch_monitor

// Why not more generic? How to handle spawning?
trait EpochDataChain
	async fn epoch_data_chain(&self, epoch, end_receiver) -> Stream

trait DataChain
	async fn data_at_index
	async fn data_stream



identity()
	Do you need to spawn a task for each



What options are there?
	adapter - fn adapt(&self, epoch, stream) -> stream
	wrapper - fn stream(&self, epoch) -> stream // This makes sense if you pass in a task-scope
	processor - fn process(&self, epoch, stream, "runner")
		// problem is that &self lifetime, particularly if processor wants to do async work inside process
		// each processor could have its own task-scope
			// gets epochs via channel
			// spawns tasks for each epoch
			// means &self lifetime is totally fine
			// 
		// difficult to build these??? need to pass in the "processor" async closure
		// what is the advantage? The running is all in one scope, so lifetime's could be guaranteed?

	runner - fn run(self, runner)
	epoch-level - same versions as above but with stream of epoch+stream


could build these up per epoch or just once?
	either way we'd FnOnce or the top level async-closures

object that has function to adapt stream (with &self)
object that has function that adapts stream, and runs the stream

note that first element on chain is not special
	so builder pattern is strange
	builder that doesn't impl the trait it self - good idea


task that has a processer
	receive epoch

how to solve problem of each witneser needing access to "processor" with atleast &self
	could have a split architecture
		generator and processor?
			the idea here is that the individual impls can decide

			could we automatically provide access inside the stream? only with lifetimes

needs to be able to run startup code asynchronously

epoch monitor pass it on startup 


could use stream trait in the DataChain trait



what adapters do we need?
	epoch
	- retry/map
		// could clone the state? or use a reference to the state
		
		ordered_map
		unordered_map

	data chain
	- safety
		- difficult because need to check full chain sequence to determine if a given block hash is in the chain
	- limit to epoch start/end
	- only current epoch
	- only active epochs
	- retry/map
		// pass in storage
	- pair (with chain)
		// request particular block instead of chaining
			// 

stream error would be output into the data member? so index and hash are still accessible
but if getter was also retrying, means has to wait forever?
	or timeout?

design choices
	epoch monitor (ready to write)
		outputs streams
		takes async-closures or some other more specific trait object
		
		outputs all active, or only current?
	data chain source - happy with design
		Is a trait or a stream
			 


	bound_by_index(data chain)
	bound_by_time(data source) / implicitly only current epoch


epoch_monitor
	.for_current_epoch(data_source)
	.witness(None, |...| async move {

	})
	.run()

let data_chain = make_data_chain(
	endpoint,
	|endpoint| async {
		endpoint.block_header(...).await
	},
	|endpoint| async {
		endpoint.block_stream(...).await
	}
);

trait DataChainSource
	type Index
	type Hash
	type Data
	type Handle
	type Stream

	async fn build() -> (Handle, Stream)

trait DataChainHandle
	type Index
	type Hash
	type Data

	async fn data_at_index(Index) -> (Hash, Data)

OR

trait DataChainSource {
	type Index
	type Hash
	type Data

	async fn data_at_index(&self, Index) -> (Hash, Data);
	async fn data_stream(&self) -> Stream<Item = (Index, Hash, Data)>;
}

trait DataChainChunkedByEpoch {
	Stream of epochs (and stream of data)
		If you need to know about old epochs use the epoch monitor handle directly
	
Requests for data about epochs will crash the engine
}


trait DataSource
trait DataChain

immediate vs retained mode
	i.e. give ability to get a stream, or just already be a stream

	having the stream is easier, but could cause problems


data_chain
	.clone()
	.safety(...)
	.chunk_by_epoch(epoch_monitor)
	.witness(storage, ...) // problem is I cannot control how the retries are run
	.buffer()
	.start()
	.await;

data_source
	.chunk_by_epoch(epoch_monitor)
	.witness(...)
	.start()
	.await;


data_chain
	
epoch_monitor
	.chunk_data_for_active_epochs
	.for_active_epochs(data_chain.safety(PolkadotSafety::Lag(2)).regularize())
	.witness(storage, |stream| {
		stream
			.map(|...| async move {
				...
			})
			.safety(PolkadotSafety::Lag(2))
			.witness()
	})
	.start()


Builder(Take Handle, and Stream?) -> (Handle, Stream)


Builder trait can add expectations to input handle and stream?

Handle that can create streams?


retry()
	able to manually re-witness block ranges, or disable ranges
		// path option to enable manually override
			witness/eth/ingress
			witness/eth/contract/stake
		// warning when no database, but don't go back
	
DataSource
	Handle and Stream

DataChain
	Handle and Stream, but with Index and Hash?


Problem: Retry needs to know epoch bounds (could give it an epoch_monitor handle)?
Would be simple to provide handle to epoch_monitor via handle?
	Actually I think it doesn't need to know the end bound?

	But it does need to know about epochs that are inactive (so it can delete old state from db)
		This clearly could only sensibly be provided through a handle or passing a handle in

So the ChunkByEpoch
	should provide
		- epoch monitor handle
		- and provide underlying data source? (each layer can modify the data source to fit)
		- per epoch
			- epoch index
			- epoch start
		- per item
			- index
			- hash
			- data

EpochMonitor, 

Should the getter expose errors?
	Could be optional?
	But should the item type of the stream always match the getter?

Either use re-emit and hide error OR show error (getters are alwys going to be able to produce errors)

how would you want non retrying map to work?




Need to be able to see errors from getters for retry?

Maybe instead of a getter you schedule particular items? So you don't have to handle errors?
For example retry() cannot provide a reasonable way to get particular items?


re-emit(number)
	// what about forgetting?

impl EpochMonitor
	// active_epochs() -> list of (active, and a handle to tell when each is not active), and a stream of new active epochs
		// Use watcher for active state?###




TODO Error handling
	// for example passing up errors from state chain?
		// Don't do this change the SCC

// If system wants timeouts

// epoch monitor gives you just epochs
	// then that is mapped with specifics?

// could have map operation to allow definition of transformers without considering errors?

// StateChainClient changes

// Idea of doing this thing for all of these

	// retrier
		// 

// so I want ability to do single try, but change the data I'm using
	// optimization only for submitting overlapping requests?


// So item of stream and getter are the same


trait DataSource
	type Data;

	async fn data_stream() -> Stream<Item=Data> 

trait DataChain: DataSource

	type Data

	async fn data_at_index(Index) -> (Hash, Data)
	async fn data_stream -> Stream<Item=(Index, Hash, Data)>

trait DataChainChunkedByEpoch
	type DataChain;
	type EpochSource;

	async fn data_stream(self) -> (DataChain, EpochSource, EpochStream<(Epoch, DataChainStream)>)
	
trait DataSourceChunkedByEpoch
	type DataSource
	type EpochSource

	async fn data_stream(self) -> (DataChain, EpochSource, EpochStream<Epoch, DataSourceStream>)

for all EpochWitnesser
	async fn run() {
		
	}


// block unsafe data

	// pull elements from stream, but don't output them until safety margin is met?

	// buffering large number of element is problematic

// get previous block for blaanceOf via the previous block hash

// simple lag safety based of same stream
	// going back means reorg

// lag safety using other stream
	// the other stream must only go back if reorged
	// also danger if base stream has items, not in the other stream
		// so the other stream skips forward
			// there was actually a reorg the other stream missed
				// but the base stream saw the reorg
	// needs to consider hashes


regularize()
	// ordered, and hashes are linked
	// check that hash chain is correct
	// you need a cache of all block hashes and indices
	// only has to go back to the eariliest block it has output?

// needed for lag_safety

// poll underlying stream to see where it is upto
	// for chain tracking





How should the witnessers retry things ideally?
	should not automatically rewitness indices already witnessed

	this leaves problem of engines disagreeing resulting in no witness

	could solve this by forcing a single compelted witness for each block (for a given category of witnessing for a particular external chain)
	or timing out witnesses that don't resolve (then the engine can safely rewitness)
	or allow authorities to remove their witness vote

also have problem of rewitnessing (when witnessing results in multiple witness extrinsics)
	could categorize witnesses (use Enum map?) (like a mask?)

retry system
	should it include lag safety?
	should it include witness submission?
	should it allow partial success?

receives stream of block headers with data
has storage of previously successful blocks
based on the blocks we have not witnessed select from those and incoming stream
don't run same block twice concurrently
record details about partial success

should the retries care about where the stream is at?
	or should it remember where the previous run got took
		i.e. remember highest block we've seen
		THIS

trait RetryStorage {
	fn set_highest_block()

	fn succeed(index)
	fn failed(Index)

	fn retry(highest_block: Index) -> Index
}

// Abilities
	Get a block to retry
	Mark block as succeeded



always have problem of rewitnessing as engine could restart, and forget on going extrinsics? Then they succeed, but we rewitness (at reorg), therefore witness a block twice
	how could you solve this?
		engine would have to know about pending extrinsics and remember the pending witnesses
			so it could know that it previous has done this, so not to start a new witness until the previous fails
		


retry doesn't go back unless it has database

what if the cfe crashes when an extrinsic is pending?
	well cannot really stop this problem, unless we store the pending extrinsics in teh retry system?
	could also have problem of if cfe crashes in middle of submitting set of extrinsics?

before extrinsics are sent, record then as having been sent
	// you'd need to know all pending, have some on_submission callback
		// which internally records hash of transaction
		// when restart, give extrinsic submit client the requests, and their pending hashes
			// thiswould be stored in the retry storage
		// you could ask the client to tell you if previous submissions succeeded or not



Build watcher system into SCC
	and submit callbacks?
		// no just have a pending struct you can output
// you can provide a system for running some code everytime the pending set changes?
// also add unlmited retries, and exponential backoff


TODO
	Watcher System for Extrinsic submission
		Callback on submission
	SCC Unlimited Retries for extrinsics
	SCC Expo backoff for extrinsics
	Epoch Watcher
	Retrier
	Storage for Retrier
	Regularize
	Detect reorgs stage (Have a callback to run in case of reorg???) // Maybe build into the lag_safety?
	Safety
		Option to stop on larger reorg
		don't want to stop, just "pause"
		witnesser should keep going?
	Pausing when SC is in maintainence?
		filter_in_maintainence() - doesn't work well with retrier (Could make it hang all data_at_index requests)????
		filter_stream_on_pause() // you'd have to build this into the retry 
		manual_override(...) // ability to filter, and insert particular block ranges (but not any data)
	Cloneable DataChain?
	Cloneable EpochDataChain?
	Latest only (data chain?) (adapter for data source)

First design EpochMonitor

EpochMonitor interface? / Must be in Arc?
	Provides stream of epochs and all active epochs
		need to be able to determine if epoch has expired.
	request future that finishes when epoch finished
	async fn active_epochs() -> Stream // provide complete futures here?
	async fn wait_until_complete(Index)// works poorly as epoch maybe inactive?
	// method to find epoch end block
	async fn wait_until_inactive(Index)

	// Internally observes EpochStarts
		// Reads in new epoch
			// and previously current epoch's end state 
	// Reads all active epochs each block
		// deactivate epochs based on that
		// also use current epoch
	

	This will be used by the data chain chunking code to end and bound the data stream
		In terms of bounding it?

--------------------------------------------------------------

// trait for signals? For testing?
	// signal is cloneable?
	// use mpsc 1 element buffer internally

// Create stream once?
// Stream contains end signals<

// How to map to include vault data?

EpochMonitor - outputs stream and all active
	- stream includes current status and active status
VaultMonitor() -> Provides stream with vault details, including ending block?
	Maybe just a cache?

chunk_by_epoch(stream)
chunk_by_vault(stream)

to_vault() adaptor?

what about deactivation of epochs?
	cache info inside SCC
	inside epoch monitor?
	use signals? from inside stream or separately required?

maybe want to drop internal stream when epoch deactivates (avoid weirdness)?

// only_participating(account_id) (epoch stream)

Consider Testability
	trait for witnesser?

data_chain
	.chunk_by_vaults(
		epoch_monitor
			.epochs()
			.only_participating()
			.vault::<...>() -> maps to (vault details, end signal with vault end block, deactivation signal)
	)
	.witness(...)
	.run()
	.await;

// want to be able to do other things than witness?
	// for sdk?

	// want to know about ingresses and egresses (so want a trait for that)?
		struct DepositWitness

	// need to split witnessers up so they are more reusable for sdk etc (and pre-witnessing)?

// difference between pre and not pre-witnessing is the retrying i.e. database? and the data source stream 

Alteratives?




EpochMonitor

	Only a stream?

only a stream
stream with additional things
trait with function to get stream

to deduplicate work of getting 


EpochStream // Has fixed set and future stream (can just use map operation)


