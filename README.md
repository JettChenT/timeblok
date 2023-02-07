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
2023-1- // Locks in the following events to 2023-1
{--1~--10 and workday} // workdays from jan 1 to jan 10
7:30am wake up to a new day
10am ~ 11am work on EvilCorp

{sun}
4pm weekly review //weekly review every sunday

--11
8am~10am Resign from EvilCorp
- Make sure you still have access to the servers


-2- // This overrides the month information from line 1.
--1
3pm~4pm Initiate operation "Hack the planet"
```

After resolving, this could be imported into your calendar:
![](media/monthlyplan.png)

### Rules 
The TimeBlock language currently recognizes three types of statements(by order of precedence):
- Event
- Occasion
- Notes
- Filters

`Occasion` is any single line that describes a point in time.
It can be a date, a time, or a date and time.

An `Event` is a line of text that starts with an `Occasion` or `Range` and is followed by text indicating the event's name.
If an Occasion is specified, the Range will start with the Occasion and assume a duration of 30 minutes.

A `Note` just a line of text, if it occurs after an Event, it is considered a note for that event, which will correspond to the 
`DESCRIPTION` field of an ics entry.

A `Range` is simply a pair of Occasions, separated by a `~`, indicating, well, a time-range.

#### Filters
`Filters` are a special type of statement that can be used to filter out dates, events, numbers in a specified range.
Filters can be nested and combined to represent complex logic and recurring events.

In the process of resolving, filters binds to the last specified `Occasion`, iterates through all possible values that fits the occasion, 
and selects those that fits the criteria for the filter.

For example, consider the following filter:
```
-2-
{--1~--10 and workday}
```
This filter bounds to the occasion `-2-`, in which the year of the occasion could be inherited from previous occasions(by default the creation date of the file)
, and the date is unspecified.
Thus, the filter will iterate through all possible dates in February. 
Since the two sub-filters are joined by an `and` clause, the filter will only select those dates that are both in the range `--1~--10` and are workdays.

The following filters are currently supported:
- Basic logic filters: `and`, `or`, `not`
- Range filters: filters all dates in a range, eg. `--1~--10` filters all dates with day value 1 to 10 in the inferred year and month
- Day-of-week filters: `workday`, `weekend`,  `sunday`, `monday`, `tuesday`, `wednesday`, `thursday`, `friday`, `saturday` (shorthand `mon` ~ `sun` is also supported)
- "Flexible date filters": basically a shorthand for range filters, eg. `--{1~10}` is equivalent to `--1~--10`

More filters are planned to be added in the future. (My current priority is to support region specific resolving of workdays based on [workalendar](https://github.com/workalendar/workalendar))
## Installation
Currently, a [Rust](https://www.rust-lang.org/) installation
is required.

Installing from cargo:
```bash
$ cargo install timeblok
```

## Usage
Exporting to file:
```bash
$ timeblok input.txt -f output.ics
```

Exporting & opening in default calendar application:
```bash
$ timeblok input.txt -o
```
