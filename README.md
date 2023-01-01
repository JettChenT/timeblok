# TimeBlok
A language for scheduling and planning calendar events that 
compiles to iCalendar (ICS).

## The Language

### Design
The TimeBlock language is a simple markup language that can be 
used on top of any plain text file, inspired by Cal Newport's
blog post [Text File Time Blocking](https://www.calnewport.com/blog/2020/03/16/text-file-time-blocking/).

The language aims to maintain a minimalistic syntax, keeping
the versatility of plain text files, while providing an interface
to the convienence of modern digital calendars via compiling
to .ics files, a file format for digital events that's barely human-readable but is supported by all calendar applications.

### Examples
#### Daily planning
This is the simplest use case
```
2023-1-1
7:30am wake up & eat beakfast
8am~11:30 work on TimeBlok
- Write Technical Documentation
2pm~6pm Study for exams
8pm~10pm Reading
- Finish an entire book
```
When compiled into an .ics file, this could be imported into your calendar.
![](./media/dayplan.png)

#### Monthly planning
```
2023-1-
--5
7:30am wake up to a new day
10am ~ 11am work on EvilCorp
- Gain root access to EvilCorp's servers
- Do not get caught

--10
8am~10am Resign from EvilCorp
- Make sure you still have access to the servers

-2-
--1
3pm~4pm Initiate operation "Hack the planet"
```

When resolved, this is equivalent to the following events, which will then be compiled to an .ics file:
```
2023-1-5 7:30~8:00 wake up to a new day
2023-1-5 10:00~11:00 work on EvilCorp
    - Gain root access to EvilCorp's servers
2023-1-10 8:00~10:00 Resign from EvilCorp
    - Make sure you still have access to the servers
2023-2-1 3:00~4:00 Initiate operation "Hack the planet"
```

... Which could be imported into your digital calendar!
![](media/monthlyplan.png)

### Rules (WIP)
(Correct me if I'm wrong about any terms, I started [Crafting Interpreters](https://craftinginterpreters.com/) a week ago)

The TimeBlock language currently recognizes three types of statements(by order of precedence):
- Event
- Occasion
- Notes

`Occasion` is any single line that describes a point in time.
It can be a date, a time, or a date and time.

An `Event` is a line of text that starts with an `Occasion` or `Range` and is followed by text indicating the event's name.
If an Occasion is specified, the Range will start with the Occasion and assume a duration of 30 minutes.

A `Note` just a line of text, if it occurs after an Event, it is considered a note for that event, which will correspond to the 
`DESCRIPTION` field of an ics entry.

A `Range` is simply a pair of Occasions, separated by a `~`, indicating, well, a time-range.



## Installation
Currently, a [Rust](https://www.rust-lang.org/) installation
is required.

Installing from cargo:
```bash
$ cargo install timeblok
```

## Usage
```bash
$ timeblok -i input.txt -o output.ics
```

