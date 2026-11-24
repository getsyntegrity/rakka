Verdict path: full replay (2966 cases); outcome DETECTED

# App replay

Build: `$TMPDIR/pekko-mutants-86196-2hvd7507/target-mailbox-id-selection-ignored/release/examples/classic-parity`

Passed: 2944/2966

Fallback rate: not reported by this build

## Failing cases

### mb-0001

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-01 open to scheduled -> first=true second=false open scheduled=true suspendCount=0 raw=2
```

stderr:

```diff
+ 
+ thread 'main' (10654074) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0002

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-02 scheduled to idle -> idle=true open scheduled=false suspendCount=0 raw=0 | again=true open scheduled=false suspendCount=0 raw=0 | reschedule=true
```

stderr:

```diff
+ 
+ thread 'main' (10654095) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0003

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-03 open to suspended and resumed -> suspend1=true suspend2=false open scheduled=false suspendCount=2 raw=8 | resume1=false open scheduled=false suspendCount=1 raw=4 resume2=true open scheduled=false suspendCount=0 raw=0 | resumeAtZero=true open scheduled=false suspendCount=0 raw=0
```

stderr:

```diff
+ 
+ thread 'main' (10654104) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0004

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-04 suspended and scheduled -> schedule=true open scheduled=true suspendCount=1 raw=6 canBe(f,f)=false canBe(f,sys)=true | resume=true open scheduled=true suspendCount=0 raw=2
```

stderr:

```diff
+ 
+ thread 'main' (10654108) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0005

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-05 open to closed -> before open scheduled=true suspendCount=2 raw=10 | close1=true closed scheduled=false suspendCount=0 raw=1 close2=false closed scheduled=false suspendCount=0 raw=1
```

stderr:

```diff
+ 
+ thread 'main' (10654122) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0006

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-06 closed ignores resume and suspend -> resume=false suspend=false closed scheduled=false suspendCount=0 raw=1
```

stderr:

```diff
+ 
+ thread 'main' (10654131) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0007

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-07 closed refuses scheduling -> schedule=false idle=true closed scheduled=false suspendCount=0 raw=1 canBe=false
```

stderr:

```diff
+ 
+ thread 'main' (10654565) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0008

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ST-08 canBeScheduledForExecution matrix -> open: FTTT withSystemMessage=true; scheduled: FTTT withSystemMessage=true; suspended: FTFT withSystemMessage=true; suspended+scheduled: FTFT withSystemMessage=true; closed: FFFF withSystemMessage=false;
```

stderr:

```diff
+ 
+ thread 'main' (10654579) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0009

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-MQ-01 default queue is FIFO and counts -> new: n=0 has=false | enqueue n=1 has=true n=2 has=true n=3 has=true | dequeue [a, b, c] then null | after n=0 has=false
```

stderr:

```diff
+ 
+ thread 'main' (10654797) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0010

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-MQ-02 default queue stays FIFO when enqueue and dequeue interleave -> first=a rest=[b, c]
```

stderr:

```diff
+ 
+ thread 'main' (10654801) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0011

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-MQ-03 cleanUp moves everything to the dead-letters queue in order -> source n=0 has=false | target n=3 has=true order [a, b, c]
```

stderr:

```diff
+ 
+ thread 'main' (10654815) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0012

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-SQ-01 system queue drains earliest first -> has=true drain=[Suspend, Resume, Terminate] | after has=false drain=[]
```

stderr:

```diff
+ 
+ thread 'main' (10654819) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0013

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-SQ-02 sentinel NoMessage closes the system queue -> drain=[Suspend] | afterSentinel has=false drain=[] (actor==null: dropped, not forwarded)
```

stderr:

```diff
+ 
+ thread 'main' (10654833) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0014

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-DL-01 watching deadLetters is not answered, watching a stopped actor is -> watcherLog=[terminated]
```

stderr:

```diff
+ 
+ thread 'main' (10654842) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0015

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-SC-01 throughput 2 on one thread interleaves two actors in blocks of two -> [r:block, r:x1, r:x2, r:y1, r:y2, r:x3, r:x4, r:y3, r:y4, r:x5, r:x6, r:y5, r:y6]
```

stderr:

```diff
+ 
+ thread 'main' (10655130) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0016

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-SC-02 throughput-deadline-time ends the turn after a slow message -> [r:block, r:sleep-x1, r:sleep-y1, r:sleep-x2, r:sleep-y2, r:sleep-x3, r:sleep-y3]
```

stderr:

```diff
+ 
+ thread 'main' (10655144) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0017

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-UT-04 control: a parent stopped after its child started waits for it, and the child dies first -> parentTerminated=true childTerminated=true deadLetters=[] log=[P:preStart, c:preStart, c:r:m1, c:postStop, P:postStop]
```

stderr:

```diff
+ 
+ thread 'main' (10655219) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0018

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-UT-08 senders racing with the installation of the real cell: nothing lost, nothing repeated, per-sender order kept -> delivered=400 distinct=400 perSenderOrdered=true preStarts=1 deadLetters=[]
```

stderr:

```diff
+ 
+ thread 'main' (10655220) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0019

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ID-01 two ids select their own queue and two actors with the same id keep separate queues -> a received=[block, m1, m2] deadLettered=[m3, m4, m5, m6]; b received=[block, m1, m2, m3, m4] deadLettered=[m5, m6]; c received=[block, m1, m2] deadLettered=[m3, m4, m5, m6];
```

stderr:

```diff
+ 
+ thread 'main' (10655233) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0020

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ID-02 an unknown id fails actorOf at the call: no actor is made and the name is free again -> actorOf: org.apache.pekko.ConfigurationException: configuration problem while creating [pekko://mbx/user/w] with dispatcher [pekko.actor.default-dispatcher] and mailbox [typo] | cause org.apache.pekko.ConfigurationException: Mailbox Type [typo] not configured | instancesStartedAfterRetryWithSameName=1
```

stderr:

```diff
+ 
+ thread 'main' (10655237) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### mb-0021

Arguments: `mailbox-case`

status:

```diff
expected exit code 0, got exit code 101
```

stdout:

```diff
- MBX-ID-03 no selection, the built-in ids and the empty id all give the unbounded FIFO queue -> w0 received=[block, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10] deadLettered=[]; w1 received=[block, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10] deadLettered=[]; w2 received=[block, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10] deadLettered=[]; w3 received=[block, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10] deadLettered=[];
```

stderr:

```diff
+ 
+ thread 'main' (10655251) panicked at rakka-actor/examples/classic-parity/mailbox_cases.rs:618:9:
+ condition not reached
+ note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### ok-user-info-lone-surrog

Arguments: `path-from-string`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
```

stderr:

```diff
+ input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)
```
