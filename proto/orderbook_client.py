import grpc
import orderbook_pb2
import orderbook_pb2_grpc
from rich.console import Console
from rich.table import Table
from time import sleep

def fetch_book_summary():
    with grpc.insecure_channel('localhost:50051') as channel:
        stub = orderbook_pb2_grpc.OrderbookAggregatorStub(channel)
        for summary in stub.BookSummary(orderbook_pb2.Empty()):
            yield summary

def get_exchange_color(exchange):
    colors = {
        "Binance": "cyan",
        "Bitstamp": "yellow",
        "Kraken": "magenta",
        "Coinbase": "green"
    }
    return colors.get(exchange, "white")

def display_table():
    console = Console()

    while True:
        try:
            console.clear()  # Clear the console for the updated table

            # Fetch data from the gRPC server
            for summary in fetch_book_summary():
                # Create the table structure
                spread_column_name = f"{summary.spread:.7f}"
                table = Table(show_header=True, header_style="bold magenta")
                table.add_column("Exchange", justify="center")
                table.add_column("Bids", justify="right", style="bright_green")
                table.add_column(spread_column_name, justify="center")
                table.add_column("Asks", justify="left", style="bright_red")
                table.add_column("Exchange", justify="center")

                # Collect all prices from bids and asks
                all_prices = {bid.price for bid in summary.bids} | {ask.price for ask in summary.asks}
                sorted_prices = sorted(all_prices, reverse=True)

                # Prepare rows for the table
                for price in sorted_prices:
                    bid = next((b for b in summary.bids if b.price == price), None)
                    ask = next((a for a in summary.asks if a.price == price), None)

                    bid_exchange_color = get_exchange_color(bid.exchange) if bid else "white"
                    ask_exchange_color = get_exchange_color(ask.exchange) if ask else "white"

                    table.add_row(
                        f"[{bid_exchange_color}]{bid.exchange}[/]" if bid else "",
                        f"{bid.amount:,.2f}" if bid else "",
                        f"{price:.7f}",
                        f"{ask.amount:,.2f}" if ask else "",
                        f"[{ask_exchange_color}]{ask.exchange}[/]" if ask else ""
                    )

                # Render the table
                console.print(table)

                # Pause briefly before clearing the table to fetch new data
                # sleep(0.05)

        except grpc.RpcError as e:
            console.print(f"[bold red]Error connecting to gRPC server: {e}[/]")
            break

if __name__ == "__main__":
    display_table()
